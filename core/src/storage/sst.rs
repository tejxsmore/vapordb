use crate::error::Result;
use crate::storage::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SSTableEntry {
    key: String,
    value: Option<Value>, // None = tombstone
    ttl: Option<u64>,     // Epoch seconds
}

#[derive(Clone)]
pub struct SSTable {
    pub map: HashMap<String, Option<Value>>,
    pub ttl_map: HashMap<String, u64>,
}

impl SSTable {
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    pub fn load(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut map = HashMap::new();
        let mut ttl_map = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let entry: SSTableEntry = match serde_json::from_str(&line) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Skipping malformed line: {e}");
                    continue;
                }
            };

            // Check TTL before inserting
            if let Some(ttl) = entry.ttl {
                if Self::current_timestamp() >= ttl {
                    // Skip expired entry entirely
                    continue;
                }
                ttl_map.insert(entry.key.clone(), ttl);
            }

            map.insert(entry.key, entry.value);
        }

        Ok(Self { map, ttl_map })
    }

    pub fn write(path: &str, map: &HashMap<String, Option<Value>>, ttl_map: &HashMap<String, u64>) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);

        for (key, value) in map {
            if let Some(ttl) = ttl_map.get(key) {
                if Self::current_timestamp() >= *ttl {
                    continue; // Skip expired
                }
            }

            let entry = SSTableEntry {
                key: key.clone(),
                value: value.clone(),
                ttl: ttl_map.get(key).cloned(),
            };
            writeln!(writer, "{}", serde_json::to_string(&entry)?)?;
        }

        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        if let Some(ttl) = self.ttl_map.get(key) {
            if Self::current_timestamp() >= *ttl {
                return None;
            }
        }

        self.map.get(key).cloned().flatten()
    }

    pub fn insert(&mut self, key: String, value: Value, ttl: Option<u64>) {
        self.map.insert(key.clone(), Some(value));
        if let Some(t) = ttl {
            self.ttl_map.insert(key, t);
        } else {
            self.ttl_map.remove(&key);
        }
    }

    pub fn delete(&mut self, key: &str) {
        self.map.insert(key.to_string(), None);
        self.ttl_map.remove(key);
    }

    pub fn size(&self) -> usize {
        self.map.len()
    }

    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            ttl_map: HashMap::new(),
        }
    }

    pub fn merge(ssts: &[SSTable]) -> SSTable {
        let mut combined_map = HashMap::new();
        let mut combined_ttl = HashMap::new();

        for sst in ssts {
            for (key, value) in &sst.map {
                combined_map.insert(key.clone(), value.clone());
            }
            for (key, ttl) in &sst.ttl_map {
                combined_ttl.insert(key.clone(), *ttl);
            }
        }

        SSTable {
            map: combined_map,
            ttl_map: combined_ttl,
        }
    }

    pub fn compact(sst1: &SSTable, sst2: &SSTable, output_path: &str) -> Result<()> {
        let merged = SSTable::merge(&[sst1.clone(), sst2.clone()]);
        SSTable::write(output_path, &merged.map, &merged.ttl_map)
    }
}
