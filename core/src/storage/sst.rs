use crate::error::Result;
use crate::storage::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct SSTableEntry {
    key: String,
    value: Option<Value>, // None represents a tombstone (deleted key)
}

pub struct SSTable {
    map: HashMap<String, Option<Value>>, // Option<Value> enables deletions (tombstones)
}

impl SSTable {
    /// Load SSTable from file
    pub fn load(path: &str) -> Result<Self> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut map = HashMap::new();

        for line in reader.lines() {
            let line = line?;
            let entry: SSTableEntry = serde_json::from_str(&line)?;
            map.insert(entry.key, entry.value);
        }

        Ok(Self { map })
    }

    /// Write SSTable to file
    pub fn write(path: &str, map: &HashMap<String, Option<Value>>) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        let mut writer = BufWriter::new(file);

        for (key, value) in map {
            let entry = SSTableEntry {
                key: key.clone(),
                value: value.clone(),
            };
            let json_line = serde_json::to_string(&entry)?;
            writeln!(writer, "{}", json_line)?;
        }

        Ok(())
    }

    /// Get value by key
    pub fn get(&self, key: &str) -> Option<Value> {
        match self.map.get(key)? {
            Some(v) => Some(v.clone()),
            None => None, // Tombstone or deleted
        }
    }

    /// Check if key exists (including tombstone)
    pub fn exists(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Insert or update a key-value pair
    pub fn insert(&mut self, key: String, value: Value) {
        self.map.insert(key, Some(value));
    }

    /// Mark a key as deleted (tombstone)
    pub fn delete(&mut self, key: &str) {
        self.map.insert(key.to_string(), None);
    }

    /// Get the number of entries in the SSTable
    pub fn size(&self) -> usize {
        self.map.len()
    }

    /// Merge multiple SSTables into one (used for compaction)
    pub fn merge(ssts: &[SSTable]) -> SSTable {
        let mut combined = HashMap::new();

        for sst in ssts {
            for (key, value) in &sst.map {
                // If the key already exists and the value is not a tombstone (None), overwrite it
                // If it's a tombstone (None), ensure the key is marked as deleted
                if let Some(existing_value) = combined.get_mut(key) {
                    if value.is_some() {
                        // If the new value is not a tombstone, we update it
                        *existing_value = value.clone();
                    }
                } else {
                    // Insert the key-value pair if it doesn't exist yet
                    combined.insert(key.clone(), value.clone());
                }
            }
        }

        SSTable { map: combined }
    }

    pub fn compact(sst1: &SSTable, sst2: &SSTable, output_path: &str) -> Result<()> {
        let mut merged_map: HashMap<String, Option<Value>> = HashMap::new(); // Use Option<Value> to handle deletions

        // Merge the two SSTables
        for (key, value) in &sst1.map {
            merged_map.insert(key.clone(), value.clone());
        }

        for (key, value) in &sst2.map {
            merged_map.insert(key.clone(), value.clone());
        }

        // Write the merged SSTable to disk
        let file = File::create(output_path)?;
        let mut writer = BufWriter::new(file);

        for (key, value) in merged_map {
            let entry = SSTableEntry { key, value };
            let json_line = serde_json::to_string(&entry)?;
            writeln!(writer, "{}", json_line)?;
        }

        Ok(())
    }
}
