use bytes::Bytes;
use futures::{SinkExt, StreamExt};
use kv::{CommandRequest, Service, ServiceInner, SledDb};
use prost::Message;
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use tracing::{debug, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // 这里需要标注类型，且要指定 Store 为 SledDb
    let service: Service<SledDb> = ServiceInner::new(SledDb::new("tmp/kvserver"))
        .fn_before_send(|res| match res.message.as_ref() {
            "" => res.message = "altered. Original message is empty.".into(),
            s => res.message = format!("altered: {}", s),
        })
        .into();
    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);

        // 使用 Framed/LengthDelimitedCodec 界定一个消息帧，自动处理消息头和消息体
        let mut stream = Framed::new(stream, LengthDelimitedCodec::new());

        let svc = service.clone();
        tokio::spawn(async move {
            while let Some(Ok(data)) = stream.next().await {
                let cmd = CommandRequest::decode(data).unwrap();
                info!("Got a new command: {:?}", cmd);
                let resp = svc.execute(cmd);
                debug!("Resp: {:?}", resp);
                let resp = resp.encode_to_vec();
                stream.send(Bytes::from(resp)).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }
}
