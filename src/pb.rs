pub mod abi;

use abi::{command_request::RequestData, *};
use bytes::Bytes;
use http::StatusCode;
use prost::Message;

use crate::KVError;

impl CommandRequest {
    /// 创建 Hset 命令
    pub fn new_hset(table: impl Into<String>, key: impl Into<String>, value: Value) -> Self {
        Self {
            request_data: Some(RequestData::Hset(Hset {
                table: table.into(),
                pair: Some(KvPair::new(key, value)),
            })),
        }
    }

    /// 创建 Hget 命令
    pub fn new_hget(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hget(Hget {
                table: table.into(),
                key: key.into(),
            })),
        }
    }

    /// 创建 Hgetall 命令
    pub fn new_hgetall(table: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hgetall(Hgetall {
                table: table.into(),
            })),
        }
    }

    /// 创建 Hdel 命令
    pub fn new_hdel(table: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            request_data: Some(RequestData::Hdel(Hdel {
                table: table.into(),
                key: key.into(),
            })),
        }
    }
}

impl KvPair {
    /// 创建新的 kvpair
    pub fn new(key: impl Into<String>, value: Value) -> Self {
        Self {
            key: key.into(),
            value: Some(value),
        }
    }
}

/// 从 String 转成 Value
impl From<String> for Value {
    fn from(s: String) -> Self {
        Self {
            value: Some(value::Value::String(s)),
        }
    }
}

/// 从 &str 转成 Value
impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self {
            value: Some(value::Value::String(s.into())),
        }
    }
}

/// 从 i64 转成 Value
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self {
            value: Some(value::Value::Integer(i)),
        }
    }
}

/// 从 f64 转成 Value
impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self {
            value: Some(value::Value::Float(f)),
        }
    }
}

/// 从 bool 转成 Value
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self {
            value: Some(value::Value::Bool(b)),
        }
    }
}

/// 从 Value 转成 CommandResponse
impl From<Value> for CommandResponse {
    fn from(v: Value) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: vec![v],
            ..Default::default()
        }
    }
}

/// 从 Vec<Value> 转成 CommandResponse
impl From<Vec<Value>> for CommandResponse {
    fn from(v: Vec<Value>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            values: v,
            ..Default::default()
        }
    }
}

/// 从 Vec<KvPair> 转成 CommandResponse
impl From<Vec<KvPair>> for CommandResponse {
    fn from(v: Vec<KvPair>) -> Self {
        Self {
            status: StatusCode::OK.as_u16() as _,
            pairs: v,
            ..Default::default()
        }
    }
}

/// 从 KVError 转成 CommandResponse
impl From<KVError> for CommandResponse {
    fn from(e: KVError) -> Self {
        let mut result = Self {
            status: StatusCode::INTERNAL_SERVER_ERROR.as_u16() as _,
            message: e.to_string(),
            values: vec![],
            pairs: vec![],
        };

        match e {
            KVError::NotFound(_, _) => result.status = StatusCode::NOT_FOUND.as_u16() as _,
            KVError::InvalidCommand(_) => result.status = StatusCode::BAD_REQUEST.as_u16() as _,
            _ => {}
        }

        result
    }
}

/// 尝试从 Value 转成 Bytes
impl TryFrom<Value> for Bytes {
    type Error = KVError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value.value {
            Some(value::Value::Binary(b)) => Ok(b),
            _ => Err(KVError::ConvertError(value, "Binary")),
        }
    }
}

/// 尝试从 Value 转成 i64
impl TryFrom<Value> for i64 {
    type Error = KVError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Integer(i)) => Ok(i),
            _ => Err(KVError::ConvertError(v, "Integer")),
        }
    }
}

/// 尝试从 Value 转成 f64
impl TryFrom<Value> for f64 {
    type Error = KVError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Float(f)) => Ok(f),
            _ => Err(KVError::ConvertError(v, "Float")),
        }
    }
}

/// 尝试从 Value 转成 bool
impl TryFrom<Value> for bool {
    type Error = KVError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.value {
            Some(value::Value::Bool(b)) => Ok(b),
            _ => Err(KVError::ConvertError(v, "Bool")),
        }
    }
}

/// 尝试从 Value 转成 Vec<u8>
impl TryFrom<Value> for Vec<u8> {
    type Error = KVError;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        let mut buf = Vec::with_capacity(v.encoded_len());

        v.encode(&mut buf)?;

        Ok(buf)
    }
}

/// 尝试从 &[u8] 转成 Value
impl TryFrom<&[u8]> for Value {
    type Error = KVError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        Ok(Value::decode(data)?)
    }
}
