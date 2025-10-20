use std::path::Path;

use sled::{Db, IVec};

use crate::{KVError, KvPair, Storage, StorageIter, Value};

#[derive(Debug)]
pub struct SledDb(Db);

impl SledDb {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self(sled::open(path).unwrap())
    }

    // 使用 prefix 模拟 table
    fn get_full_key(table: &str, key: &str) -> String {
        format!("{}:{}", table, key)
    }

    // 遍历 table 的 key 时，我们直接把 prefix: 当成 table
    fn get_table_prefix(table: &str) -> String {
        format!("{}:", table)
    }
}

impl Storage for SledDb {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, KVError> {
        let name = SledDb::get_full_key(table, key);

        self.0.get(name)?.map(|v| v.as_ref().try_into()).transpose()
    }

    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, KVError> {
        let name = SledDb::get_full_key(table, &key);
        let data: Vec<u8> = value.try_into()?;

        self.0
            .insert(name, data)?
            .map(|v| v.as_ref().try_into())
            .transpose()
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool, KVError> {
        let name = SledDb::get_full_key(table, key);

        Ok(self.0.contains_key(name)?)
    }

    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, KVError> {
        let name = SledDb::get_full_key(table, key);

        self.0
            .remove(name)?
            .map(|v| v.as_ref().try_into())
            .transpose()
    }

    fn get_all(&self, table: &str) -> Result<Vec<KvPair>, KVError> {
        let prefix = SledDb::get_table_prefix(table);

        Ok(self.0.scan_prefix(prefix).map(|v| v.into()).collect())
    }

    fn get_iter(&self, table: &str) -> Result<impl Iterator<Item = KvPair>, KVError> {
        let prefix = SledDb::get_table_prefix(table);

        Ok(StorageIter::new(self.0.scan_prefix(prefix)))
    }
}

impl From<Result<(IVec, IVec), sled::Error>> for KvPair {
    fn from(v: Result<(IVec, IVec), sled::Error>) -> Self {
        match v {
            Ok((k, v)) => match v.as_ref().try_into() {
                Ok(v) => KvPair::new(ivec_to_key(k.as_ref()), v),
                Err(_) => KvPair::default(),
            },
            _ => KvPair::default(),
        }
    }
}

fn ivec_to_key(ivec: &[u8]) -> &str {
    let s = str::from_utf8(ivec).unwrap();
    let mut iter = s.split(":");

    iter.next();
    iter.next().unwrap()
}
