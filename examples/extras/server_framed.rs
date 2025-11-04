use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化 tracing
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);

        // LengthDelimitedCodec 默认 4 字节长度
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

        tokio::spawn(async move {
            while let Some(Ok(data)) = stream.next().await {
                // 接收到的消息只包含消息主体，不包含长度
                info!("Got: {}", String::from_utf8_lossy(&data));
                // 发送的消息也需要发送主体，不需要提供长度，Framed/LengthDelimitedCodec 会自动处理并添加
                stream.send(Bytes::from("Goodbye world!")).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
