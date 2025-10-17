//! deprecated_client.rs
//! 依赖于外部 crate：prost_stream

use kv::{CommandRequest, CommandResponse};
use prost_stream::AsyncStream;
use tokio::net::TcpStream;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    // 连接服务器
    let addr = "127.0.0.1:9527";
    let stream = TcpStream::connect(addr).await?;

    let mut client = AsyncStream::new(stream);

    info!("Connected to {}", addr);

    // 生成 Hset 命令
    let cmd = CommandRequest::new_hset("table1", "hello", "world".to_string().into());

    // 发送 Hset 命令
    client.send(&cmd).await?;

    // 接收响应
    let data: CommandResponse = client.recv().await?;

    info!("Got response {:?}", data);

    Ok(())
}
