use interprocess::local_socket::{
    GenericNamespaced,
    tokio::prelude::*,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::env;
use std::io::{self, Read};


#[tokio::main]
async fn main() -> anyhow::Result<()> {

    let action = env::args()
        .nth(1)
        .unwrap_or("claude".into());


    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input)?;

    // 解析 stdin JSON，提取 hook_event_name 作为日志文件名
    let payload: serde_json::Value = serde_json::from_str(&stdin_input)
        .expect("stdin must be valid JSON");


    let message = serde_json::json!({
        "action": action,
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
    println!("{}", serde_json::to_string_pretty(&response)?);

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
            "action": "claude",
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
    }
}