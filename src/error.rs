use crate::Value;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum KVError {
    #[error("Not found for table: {0}, key: {1}")]
    NotFound(String, String),

    #[error("Command is invalid: `{0}`")]
    InvalidCommand(String),

    #[error("Cannot convert value {0:?} to {1}")]
    ConvertError(Value, &'static str),

    #[error("Cannot process command {0} with table: {1}, key: {2}. Error: {3}")]
    StorageError(&'static str, String, String, String),

    #[error("Failed to encode protobuf message")]
    EncodeError(#[from] prost::EncodeError),

    #[error("Failed to decode protobuf message")]
    DecodeError(#[from] prost::DecodeError),

    #[error("Internal error: {0}")]
    InternalError(String),
}
