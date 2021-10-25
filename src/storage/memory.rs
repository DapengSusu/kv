use dashmap::{DashMap, mapref::one::Ref};
use crate::{KvPair, Storage, Value};

/// 使用 DashMap 构建的 MemTable，实现了 Storage trait
#[derive(Debug, Clone, Default)]
pub struct MemTable {
    tables: DashMap<String, DashMap<String, Value>>
}

impl MemTable {
    /// 创建一个缺省的 MemTable
    pub fn new() -> Self {
        Self::default()
    }

    /// 如果名为 name 的 hash table 不存在，则创建，否则返回
    fn get_or_create_table(&self, name: &str) -> Ref<String, DashMap<String, Value>> {
        match self.tables.get(name) {
            Some(table) => table,
            None => self.tables.entry(name.into()).or_default().downgrade()
        }
    }
}

impl Storage for MemTable {
    fn get(&self, table: &str, key: &str) -> Result<Option<Value>, crate::KVError> {
        let table = self.get_or_create_table(table);

        Ok(table.get(key).map(|v| v.value().clone()))
    }

    fn set(&self, table: &str, key: String, value: Value) -> Result<Option<Value>, crate::KVError> {
        let table = self.get_or_create_table(table);

        Ok(table.insert(key, value))
    }

    fn contains(&self, table: &str, key: &str) -> Result<bool, crate::KVError> {
        let table = self.get_or_create_table(table);

        Ok(table.contains_key(key))
    }

    fn del(&self, table: &str, key: &str) -> Result<Option<Value>, crate::KVError> {
        let table = self.get_or_create_table(table);

        Ok(table.remove(key).map(|(_, v)| v))
    }

    fn get_all(&self, table: &str) -> Result<Vec<crate::KvPair>, crate::KVError> {
        let table = self.get_or_create_table(table);

        Ok(
            table
                .iter()
                .map(|v|KvPair::new(v.key(), v.value().clone()))
                .collect()
        )
    }

    fn get_iter(&self, _table: &str) -> Result<Box<dyn Iterator<Item = crate::KvPair>>, crate::KVError> {
        todo!() // TODO
    }
}
