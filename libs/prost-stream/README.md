# prost-stream

Read protobuf messages from a Stream

# Examples

## Stream

```rust
use prost_stream::Stream;
use std::net::TcpListener;
use std::net::TcpStream;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ping {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pong {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

fn main() -> anyhow::Result<()> {    
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;

    std::thread::spawn(move || {
        let (stream, _) = listener.accept()?;
        let mut stream = Stream::new(stream);

        let _msg: Ping = stream.recv()?;
        stream.send(&Pong::default())?;

        anyhow::Result::<()>::Ok(())
    });

    let client = TcpStream::connect(addr)?;
    let mut client = Stream::new(client);

    client.send(&Ping::default())?;
    let pong: Pong = client.recv()?;

    assert_eq!(pong, Pong::default());

    Ok(())
}
```

## AsyncStream

With `async` feature enabled, you can use AsyncStream

```rust
use prost_stream::AsyncStream;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Ping {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Pong {
    #[prost(uint64, tag = "1")]
    pub id: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        let (stream, _) = listener.accept().await?;
        let mut stream = AsyncStream::new(stream);
        let _msg: Ping = stream.recv().await?;
        stream.send(&Pong::default()).await?;

        anyhow::Result::<()>::Ok(())
    });

    let client = TcpStream::connect(addr).await?;
    let mut client = AsyncStream::new(client);

    client.send(&Ping::default()).await?;
    let pong: Pong = client.recv().await?;

    assert_eq!(pong, Pong::default());

    Ok(())
}
```
