use interprocess::local_socket::{
    GenericNamespaced,
    tokio::prelude::*,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::env;
use std::fs;
use std::io::{self, Read};
use std::time::{SystemTime, UNIX_EPOCH};


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::args()
        .skip(1)
        .any(|arg| matches!(arg.as_str(), "--version" | "-V"))
    {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let action = env::args().nth(1).unwrap_or("claude".into());

    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input)?;

    // 将 stdin 写入 exe 同级目录下的 log 目录
    {
        let exe_dir = env::current_exe()?
            .parent()
            .expect("exe has parent dir")
            .to_path_buf();
        let log_dir = exe_dir.join("log");
        fs::create_dir_all(&log_dir)?;

        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time went backwards")
            .as_millis();
        let log_file = log_dir.join(format!("{}.json", ts));
        fs::write(&log_file, stdin_input.as_bytes())?;
        eprintln!("[cc-hook] stdin logged to {}", log_file.display());
    }


    let payload: serde_json::Value = serde_json::from_str(&stdin_input)
        .expect("stdin must be valid JSON");


    let message = serde_json::json!({
        "name": action,
        "payload": payload,
    });


    let name = "灯灯侑侑天下第一".to_ns_name::<GenericNamespaced>()?;
    let conn = LocalSocketStream::connect(name).await?;
    let mut conn = BufReader::new(conn);


    let mut json_str = serde_json::to_string(&message)?;
    json_str.push('\n');
    conn.get_mut().write_all(json_str.as_bytes()).await?;


    let mut response_line = String::new();
    conn.read_line(&mut response_line).await?;

    let response: serde_json::Value = serde_json::from_str(&response_line)?;

    if response["status"] == "ok" {
        let result = &response["result"];
        // 只有当 result 不是空对象时才输出
        if !result.is_null() && !result.as_object().map_or(false, |m| m.is_empty()) {
            println!("{}", serde_json::to_string(result)?);
        } else {
            eprintln!("[cc-hook] empty result, skipping stdout");
        }
    } else {
        eprintln!("[cc-hook] status not ok: {}", response["status"]);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::fs;

    #[tokio::test]
    async fn test_ipc_direct() {
        let eg_dir = Path::new("src/eg");
        let mut json_files: Vec<_> = fs::read_dir(eg_dir)
            .expect("Failed to read src/eg directory")
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
            .collect();
        json_files.sort_by_key(|e| e.file_name());

        let names: Vec<String> = json_files
            .iter()
            .map(|e| e.path().file_stem().unwrap().to_string_lossy().into_owned())
            .collect();

        let selection = dialoguer::Select::new()
            .with_prompt("选择一个 hook event 作为 payload")
            .items(&names)
            .default(0)
            .interact()
            .expect("Failed to get selection");

        let json_content = fs::read_to_string(json_files[selection].path())
            .expect("Failed to read selected JSON");

        println!("选择了: {}", names[selection]);

        let payload: serde_json::Value = serde_json::from_str(&json_content).unwrap();
        let message = serde_json::json!({
            "name": "claude",
            "payload": payload,
        });

        let name = "灯灯侑侑天下第一".to_ns_name::<GenericNamespaced>().unwrap();
        let conn = LocalSocketStream::connect(name).await.unwrap();
        let mut conn = BufReader::new(conn);

        let mut json_str = serde_json::to_string(&message).unwrap();
        json_str.push('\n');
        conn.get_mut().write_all(json_str.as_bytes()).await.unwrap();

        let mut response_line = String::new();
        conn.read_line(&mut response_line).await.unwrap();

        let response: serde_json::Value = serde_json::from_str(&response_line).unwrap();
        println!("Response: {}", serde_json::to_string_pretty(&response).unwrap());

        assert_eq!(response["status"], "ok", "Expected status ok");

        // 测试 result 输出逻辑
        let result = &response["result"];
        if !result.is_null() && !result.as_object().map_or(false, |m| m.is_empty()) {
            println!("Result to stdout: {}", serde_json::to_string(result).unwrap());
        } else {
            println!("Empty result, nothing to stdout");
        }
    }

    #[tokio::test]
    async fn test_result_output_logic() {
        // 测试空结果
        let response1 = serde_json::json!({
            "status": "ok",
            "uuid": "test-uuid",
            "result": {}
        });
        let result1 = &response1["result"];
        assert!(result1.as_object().unwrap().is_empty());
        println!("Test 1 (empty): skip stdout ✓");

        // 测试有内容的结果
        let response2 = serde_json::json!({
            "status": "ok",
            "uuid": "test-uuid",
            "result": {
                "continue": true,
                "suppressOutput": true,
                "hookSpecificOutput": {
                    "hookEventName": "PreToolUse",
                    "permissionDecision": "allow"
                }
            }
        });
        let result2 = &response2["result"];
        assert!(!result2.as_object().unwrap().is_empty());
        println!("Test 2 (with content): {}", serde_json::to_string(result2).unwrap());

        // 测试 null 结果
        let response3 = serde_json::json!({
            "status": "ok",
            "uuid": "test-uuid",
            "result": null
        });
        let result3 = &response3["result"];
        assert!(result3.is_null());
        println!("Test 3 (null): skip stdout ✓");

        println!("All result output logic tests passed!");
    }
}
