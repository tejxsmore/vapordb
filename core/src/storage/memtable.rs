use crate::error::{VaporDBError, Result};
use crate::storage::{Storage, Value};
use serde_json;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::RwLock;

pub struct MemTable {
    pub map: RwLock<HashMap<String, Value>>,
}

impl MemTable {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }

    pub fn len(&self) -> usize {
        self.map.read().unwrap().len()
    }

    pub fn clear(&self) {
        self.map.write().unwrap().clear();
    }

    pub fn flush_to_sstable(&self, path: &str) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        let map = self.map.read().unwrap();
        for (key, value) in map.iter() {
            let serialized_value = serde_json::to_string(value)?;
            writeln!(writer, "{}\t{}", key, serialized_value)?;
        }
        Ok(())
    }

    // List operations
    pub fn lpush(&self, key: String, value: String) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let list = map.entry(key.clone()).or_insert(Value::List(vec![]));

        if let Value::List(vec) = list {
            vec.insert(0, value);
        } else {
            return Err(VaporDBError::TypeMismatch("Expected List".into()));
        }
        Ok(())
    }

    pub fn rpush(&self, key: String, value: String) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let list = map.entry(key.clone()).or_insert(Value::List(vec![]));

        if let Value::List(vec) = list {
            vec.push(value);
        } else {
            return Err(VaporDBError::TypeMismatch("Expected List".into()));
        }
        Ok(())
    }

    pub fn lpop(&self, key: String) -> Result<Option<String>> {
        let mut map = self.map.write().unwrap();
        if let Some(Value::List(vec)) = map.get_mut(&key) {
            if !vec.is_empty() {
                return Ok(Some(vec.remove(0)));
            }
        }
        Ok(None)
    }

    pub fn rpop(&self, key: String) -> Result<Option<String>> {
        let mut map = self.map.write().unwrap();
        if let Some(Value::List(vec)) = map.get_mut(&key) {
            if !vec.is_empty() {
                return Ok(Some(vec.pop().unwrap()));
            }
        }
        Ok(None)
    }

    pub fn lrange(&self, key: String, start: usize, end: usize) -> Result<Vec<String>> {
        let map = self.map.read().unwrap();
        if let Some(Value::List(vec)) = map.get(&key) {
            let start = start.min(vec.len());
            let end = end.min(vec.len());
            Ok(vec[start..end].to_vec())
        } else {
            Err(VaporDBError::TypeMismatch("Expected List".into()))
        }
    }

    // Set operations
    pub fn sadd(&self, key: String, value: String) -> Result<()> {
        let mut map = self.map.write().unwrap();
        let set = map.entry(key.clone()).or_insert(Value::Set(HashSet::new()));

        if let Value::Set(set) = set {
            set.insert(value);
        } else {
            return Err(VaporDBError::TypeMismatch("Expected Set".into()));
        }
        Ok(())
    }

    pub fn srem(&self, key: String, value: String) -> Result<()> {
        let mut map = self.map.write().unwrap();
        if let Some(Value::Set(set)) = map.get_mut(&key) {
            set.remove(&value);
            Ok(())
        } else {
            Err(VaporDBError::TypeMismatch("Expected Set".into()))
        }
    }

    pub fn smembers(&self, key: String) -> Result<HashSet<String>> {
        let map = self.map.read().unwrap();
        if let Some(Value::Set(set)) = map.get(&key) {
            Ok(set.clone())
        } else {
            Err(VaporDBError::TypeMismatch("Expected Set".into()))
        }
    }
}

impl Storage for MemTable {
    fn get(&self, key: &str) -> Result<Option<Value>> {
        Ok(self.map.read().unwrap().get(key).cloned())
    }

    fn set(&self, key: String, value: Value) -> Result<()> {
        self.map.write().unwrap().insert(key, value);
        Ok(())
    }

    fn del(&self, key: &str) -> Result<()> {
        self.map.write().unwrap().remove(key);
        Ok(())
    }

    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.map.read().unwrap().contains_key(key))
    }

    fn keys(&self) -> Result<Vec<String>> {
        Ok(self.map.read().unwrap().keys().cloned().collect())
    }
}