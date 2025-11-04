use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use kv::{CommandRequest, CommandResponse};
use prost::Message;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    // 连接服务器
    let addr = "127.0.0.1:9527";
    let stream = TcpStream::connect(addr).await?;
    info!("Connected to {}", addr);

    // 使用 Framed/LengthDelimitedCodec 界定一个消息帧，自动处理消息头和消息体
    let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

    // 生成 Hset 命令
    let cmd = CommandRequest::new_hset("table1", "hello", "world".to_string().into());

    // 发送 Hset 命令
    stream.send(Bytes::from(cmd.encode_to_vec())).await?;

    if let Some(Ok(data)) = stream.next().await {
        let resp = CommandResponse::decode(data).unwrap();
        info!("Got response {:?}", resp);
    }

    Ok(())
}
