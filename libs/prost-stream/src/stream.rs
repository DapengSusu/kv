use prost::Message;
use std::io::Read;
use std::io::Write;
use thiserror::Error;
#[cfg(feature = "async")]
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("prost decode error: {0}")]
    ProstDecodeError(#[from] prost::DecodeError),
    #[error("prost encode error: {0}")]
    ProstEncodeError(#[from] prost::EncodeError),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Stream<T> {
    stream: T,
    buf: Vec<u8>,
    send_buf: Vec<u8>,
}

impl<T: Read + Write> Stream<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            buf: vec![0; 1024],
            send_buf: Vec::with_capacity(1024),
        }
    }

    pub fn into_inner(self) -> T {
        self.stream
    }

    pub fn send(&mut self, msg: &impl Message) -> Result<()> {
        let buf = &mut self.send_buf;
        buf.clear();
        let sz = msg.encoded_len() + 10;
        buf.reserve(sz);

        msg.encode_length_delimited(buf)?;
        self.stream.write_all(buf)?;
        Ok(())
    }

    pub fn recv<M: Message + Default>(&mut self) -> Result<M> {
        let buf = &mut self.buf;
        let stream = &mut self.stream;

        // protobuf 消息的长度信息最少占有 1 byte, 最多占有 10 bytes
        // 当消息本身的长度小于 128 时占用 1 byte
        stream.read_exact(&mut buf[..1])?;

        match prost::decode_length_delimiter(&buf[..1]) {
            Ok(sz) => {
                if sz > buf.len() {
                    buf.resize(sz, 0);
                }
                stream.read_exact(&mut buf[..sz])?;
                Ok(M::decode(&buf[..sz])?)
            }
            Err(_) => {
                // protobuf 消息的长度信息最少占有 1 byte, 最多占有 10 bytes
                stream.read_exact(&mut buf[1..10])?;
                let sz = prost::decode_length_delimiter(&buf[..10])?;
                let delimiter_len = prost::length_delimiter_len(sz);
                let idx = delimiter_len;
                let left = sz - (10 - idx);

                if 10 + left > buf.len() {
                    buf.resize(10 + left, 0);
                }

                stream.read_exact(&mut buf[10..left])?;
                Ok(M::decode(&buf[idx..idx + sz])?)
            }
        }
    }
}

#[cfg(feature = "async")]
pub struct AsyncStream<T> {
    stream: T,
    buf: Vec<u8>,
    send_buf: Vec<u8>,
}

#[cfg(feature = "async")]
impl<T: AsyncReadExt + AsyncWriteExt + Unpin> AsyncStream<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            buf: vec![0u8; 1024],
            send_buf: Vec::with_capacity(1024),
        }
    }

    pub fn into_inner(self) -> T {
        self.stream
    }

    pub async fn send(&mut self, msg: &impl Message) -> Result<()> {
        let buf = &mut self.send_buf;
        buf.clear();
        let sz = msg.encoded_len() + 10;
        buf.reserve(sz);

        msg.encode_length_delimited(buf)?;

        self.stream
            .write_all(buf) // &msg.encode_length_delimited_to_vec()
            .await
            .map_err(Into::into)
    }

    pub async fn recv<M: Message + Default>(&mut self) -> Result<M> {
        let buf = &mut self.buf;
        let stream = &mut self.stream;

        // protobuf 消息的长度信息最少占有 1 byte, 最多占有 10 bytes
        // 当消息本身的长度小于 128 时占用 1 byte
        stream.read_exact(&mut buf[..1]).await?;

        match prost::decode_length_delimiter(&buf[..1]) {
            Ok(sz) => {
                if sz > buf.len() {
                    buf.resize(sz, 0);
                }
                stream.read_exact(&mut buf[..sz]).await?;
                Ok(M::decode(&buf[..sz])?)
            }
            Err(_) => {
                // protobuf 消息的长度信息最少占有 1 byte, 最多占有 10 bytes
                stream.read_exact(&mut buf[1..10]).await?;
                let sz = prost::decode_length_delimiter(&buf[..10])?;
                let delimiter_len = prost::length_delimiter_len(sz);
                let idx = delimiter_len;
                let left = sz - (10 - idx);

                if 10 + left > buf.len() {
                    buf.resize(10 + left, 0);
                }

                stream.read_exact(&mut buf[10..left]).await?;
                Ok(M::decode(&buf[idx..idx + sz])?)
            }
        }
    }
}
