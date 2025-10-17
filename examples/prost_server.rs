//! prost_server.rs
//! 仅用于测试 `async_prost::AsyncProstStream` 功能

use std::sync::Arc;

use async_prost::AsyncProstStream;
use bytes::Bytes;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use prost::Message;
use tokio::net::TcpListener;
use tracing::{debug, info};

#[derive(Clone, PartialEq, Message)]
pub struct Request {
    #[prost(uint64, tag = "1")]
    tag: u64,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: Bytes,
}

impl Request {
    pub fn new(data: Bytes) -> Self {
        Request { tag: 0, data }
    }

    pub fn set_tag(&mut self, tag: usize) {
        self.tag = tag as u64;
    }

    pub fn get_tag(&self) -> usize {
        self.tag as usize
    }
}

#[derive(Clone, PartialEq, Message)]
pub struct Response {
    #[prost(uint64, tag = "1")]
    tag: u64,
    #[prost(bytes = "bytes", tag = "2")]
    pub data: Bytes,
}

impl Response {
    pub fn get_tag(&self) -> usize {
        self.tag as usize
    }
}

impl From<Request> for Response {
    fn from(r: Request) -> Response {
        Response {
            tag: r.tag,
            data: r.data,
        }
    }
}

/// Service 数据结构
pub struct Service<Store = MemStore> {
    inner: Arc<ServiceInner<Store>>,
}

impl<Store> Clone for Service<Store> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Store: Storage> Service<Store> {
    pub fn new(store: Store) -> Self {
        Self {
            inner: Arc::new(ServiceInner { store }),
        }
    }

    pub fn execute(&self, req: Request) -> Response {
        debug!("Got request: {:?}", req.get_tag());
        // TODO: 发送 on_received 事件

        let res: Response = req.into();
        debug!("Executed response: {:?}", res.get_tag());
        // TODO: 发送 on_executed 事件

        res
    }
}

/// Service 内部数据结构
#[allow(dead_code)]
pub struct ServiceInner<Store> {
    store: Store,
}

#[derive(Debug, Default, Eq, PartialEq, Hash)]
pub struct Tag(u64);

impl From<Tag> for u64 {
    fn from(tag: Tag) -> Self {
        tag.0
    }
}

impl From<Tag> for usize {
    fn from(tag: Tag) -> Self {
        tag.0 as usize
    }
}

/// 数据存储
pub trait Storage {
    fn set(&mut self, tag: Tag, info: String) -> anyhow::Result<Option<String>>;
    fn get(&self, tag: &Tag) -> anyhow::Result<Option<String>>;
}

/// 实现 Storage trait
#[derive(Debug, Default)]
pub struct MemStore {
    map: DashMap<Tag, String>,
}

impl MemStore {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: DashMap::with_capacity(capacity),
        }
    }
}

impl Storage for MemStore {
    fn set(&mut self, tag: Tag, info: String) -> anyhow::Result<Option<String>> {
        Ok(self.map.insert(tag, info))
    }

    fn get(&self, tag: &Tag) -> anyhow::Result<Option<String>> {
        Ok(self.map.get(tag).map(|v| v.value().clone()))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let service = Service::new(MemStore::new());
    let addr = "127.0.0.1:9527";
    let listener = TcpListener::bind(addr).await?;
    info!("Start listening on {}", addr);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);
        let svc = service.clone();
        // let mut stream = AsyncStream::new(stream);

        tokio::spawn(async move {
            let mut stream = AsyncProstStream::<_, Request, Response, _>::from(stream).for_async();
            while let Some(Ok(cmd)) = stream.next().await {
                info!("Got a new command: {:?}", cmd);
                let res = svc.execute(cmd);
                stream.send(res).await.unwrap();
            }
            // while let Ok(cmd) = stream.recv().await {
            //     info!("Got a new command: {:?}", cmd);
            //     let res = svc.execute(cmd);
            //     stream.send(&res).await.unwrap();
            // }
            info!("Client {:?} disconnected", addr);
        });
    }
}
