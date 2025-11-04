use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use kv::{CommandRequest, CommandResponse};
use prost::Message;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);

        tokio::spawn(async move {
            let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

            while let Some(Ok(data)) = stream.next().await {
                let cmd = CommandRequest::decode(data).unwrap();
                info!("Got a new command: {:?}", cmd);
                // 创建一个 404 response 返回给客户端
                let resp = CommandResponse {
                    status: 404,
                    message: "Not found".to_string(),
                    ..Default::default()
                }
                .encode_to_vec();

                stream.send(Bytes::from(resp)).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
