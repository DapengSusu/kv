use prost_stream::AsyncStream;
use kv::CommandResponse;
use tokio::net::TcpListener;
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
            let mut stream = AsyncStream::new(stream);

            while let Ok(msg) = stream.recv::<CommandResponse>().await {
                info!("Got a new command: {:?}", msg);
                // 创建一个 404 response 返回给客户端
                let resp = CommandResponse {
                    status: 404,
                    message: "Not found".to_string(),
                    ..Default::default()
                };

                stream.send(&resp).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
