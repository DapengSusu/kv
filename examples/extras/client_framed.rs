use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:9527";
    let stream = TcpStream::connect(addr).await?;
    info!("Connected to {}", addr);

    let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

    stream.send(Bytes::from("Hello world!")).await?;

    // 接收服务器返回的数据
    if let Some(Ok(data)) = stream.next().await {
        info!("Got: {}", String::from_utf8_lossy(&data));
    }

    Ok(())
}
