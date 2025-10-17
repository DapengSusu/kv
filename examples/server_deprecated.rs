//! deprecated_server.rs
//! 依赖于外部 crate：prost_stream

use kv::{MemTable, Service};
use prost_stream::AsyncStream;
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let service = Service::new(MemTable::new());
    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);
        let svc = service.clone();
        let mut stream = AsyncStream::new(stream);

        tokio::spawn(async move {
            while let Ok(cmd) = stream.recv().await {
                info!("Got a new command: {:?}", cmd);
                let res = svc.execute(cmd);
                stream.send(&res).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
