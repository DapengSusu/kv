// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

#![cfg_attr(feature="async", doc=include_str!("../README.md"))]

mod stream;
pub use stream::*;

// cargo test --all-features 

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn it_works() -> anyhow::Result<()> {
        use std::net::TcpListener;
        use std::net::TcpStream;

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;

        std::thread::spawn(move || {
            let (stream, _) = listener.accept()?;
            let mut stream = Stream::new(stream);

            let msg: Ping = stream.recv()?;
            assert_eq!(msg, Ping{ id: 1234 });
            stream.send(&Pong{ id: 9527 })?;

            let msg: Ping = stream.recv()?;
            assert_eq!(msg, Ping{ id: 4321 });
            stream.send(&Pong{ id: 7259 })?;

            anyhow::Result::<()>::Ok(())
        });

        let client = TcpStream::connect(addr)?;
        let mut client = Stream::new(client);

        client.send(&Ping{ id: 1234 })?;
        let pong: Pong = client.recv()?;
        assert_eq!(pong, Pong{ id: 9527 });

        client.send(&Ping{ id: 4321 })?;
        let pong: Pong = client.recv()?;
        assert_eq!(pong, Pong{ id: 7259 });

        Ok(())
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_async() -> anyhow::Result<()> {
        use tokio::net::TcpListener;
        use tokio::net::TcpStream;

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let (stream, _) = listener.accept().await?;
            let mut stream = AsyncStream::new(stream);

            let msg: Ping = stream.recv().await?;
            assert_eq!(msg, Ping{ id: 1234 });
            stream.send(&Pong{ id: 9527 }).await?;

            let msg: Ping = stream.recv().await?;
            assert_eq!(msg, Ping{ id: 4321 });
            stream.send(&Pong{ id: 7259 }).await?;

            anyhow::Result::<()>::Ok(())
        });

        let client = TcpStream::connect(addr).await?;
        let mut client = AsyncStream::new(client);

        client.send(&Ping{ id: 1234 }).await?;
        let pong: Pong = client.recv().await?;
        assert_eq!(pong, Pong{ id: 9527 });

        client.send(&Ping{ id: 4321 }).await?;
        let pong: Pong = client.recv().await?;
        assert_eq!(pong, Pong{ id: 7259 });

        Ok(())
    }
}
