# cc-hook-core

基于本地命名管道的 IPC 通信框架，用于 Agent 与服务端之间的 JSON 请求/响应交互。

## TODO

- [ ] Claude Code 适配
- [ ] Codex 适配

## 快速开始

```bash
# 启动服务端
cargo run --bin server

# 发送请求
cargo run --bin client -- claude < src/xxx.json
```

## 协议

客户端与服务端通过本地命名管道通信，每行一个完整 JSON。

### 请求格式

```json
{"action": "<action>", "payload": <任意 JSON>}
```

### 响应格式

根据不同 agent，`stdout` 内容略有不同：

```json
{"status": "ok", "stdout": <任意 JSON>}
```

## 日志

每次请求会自动将 stdin 原始内容保存到 `logs/` 目录；当响应 `status` 为 `ok` 且有可输出内容时，也会把要输出到 stdout 的内容保存一份，文件名都使用时间戳（毫秒）。

## 测试

```bash
# 测试会自动启动 server，用 xxx.json 作为输入
cargo test -- --nocapture
```

测试默认读取同目录下的 `xxx.json`，使用 `claude` 作为默认 action。
