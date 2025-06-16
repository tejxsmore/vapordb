use crate::command::Command;
use crate::error::{VaporDBError, Result};
use crate::storage::sst::SSTable;
use crate::storage::{memtable::MemTable, Storage, Value};
use crate::ttl::ExpirationTable;
use crate::wal::wal::{LogEntry, WriteAheadLog};
use serde_json;
use std::sync::{Arc, Mutex};

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

pub struct VaporDB {
    storage: Arc<MemTable>,
    ttl: Arc<ExpirationTable>,
    wal: WriteAheadLog,
    sstables: Vec<SSTable>,
    sst_dir: PathBuf,       // directory where SSTs are stored
    flush_threshold: usize, // flush when this many keys are in MemTable
}

impl VaporDB {
    pub fn new_with_persistence(wal_path: &str) -> Result<Self> {
        let storage = Arc::new(MemTable::new());
        let ttl = Arc::new(ExpirationTable::new());
        let wal = WriteAheadLog::new(wal_path)?;

        // Load SSTables
        let sst_dir = PathBuf::from("sstables");
        std::fs::create_dir_all(&sst_dir)?;

        let mut sstables = vec![];
        for entry in std::fs::read_dir(&sst_dir)? {
            let path = entry?.path();
            if path.extension().map(|ext| ext == "sst").unwrap_or(false) {
                let sst = SSTable::load(path.to_str().unwrap())?;
                sstables.push(sst);
            }
        }

        for entry in wal.load_entries()? {
            match entry {
                LogEntry::Set(k, v) => {
                    storage.set(k, Value::String(v))?;
                }
                LogEntry::Del(k) => {
                    storage.del(&k)?;
                }
            }
        }

        let vapor_db = Self {
            storage,
            wal,
            ttl,
            sstables,
            sst_dir,
            flush_threshold: 1000,
        };

        Ok(vapor_db)
    }

    pub fn memtable(&self) -> Arc<MemTable> {
        Arc::clone(&self.storage)
    }

    pub fn expiration_table(&self) -> Arc<ExpirationTable> {
        Arc::clone(&self.ttl)
    }

    pub fn sstable(&self) -> Option<Arc<Mutex<SSTable>>> {
        self.sstables.get(0).cloned().map(|sst| Arc::new(Mutex::new(sst)))
    }

    pub fn start_ttl_daemon(db: Arc<Mutex<Self>>) {
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            if let Ok(db) = db.lock() {
                db.clean_expired_keys();
            }
        });
    }

    pub fn clean_expired_keys(&self) {
        let expired_keys = self.ttl.get_expired_keys();
        for key in expired_keys {
            let _ = self.storage.del(&key);
            self.ttl.remove(&key);
        }
    }

    pub fn start_background_compaction(&mut self) -> Result<()> {
        self.sstables.sort_by_key(|sst| sst.size());

        let sstables_len = self.sstables.len();
        if sstables_len >= 2 {
            let sst1 = &self.sstables[0];
            let sst2 = &self.sstables[1];

            let timestamp = chrono::Utc::now().timestamp();
            let path = self.sst_dir.join(format!("compact_{}.sst", timestamp));

            // Handle compaction result
            if let Err(e) = SSTable::compact(sst1, sst2, path.to_str().unwrap()) {
                return Err(VaporDBError::CompactionFailed(e.to_string()));
            } else {
                println!("Compaction successful!");
            }

            // Remove old SSTables from memory or disk and Reload the compacted SSTable
            self.sstables.drain(0..2);
            self.sstables.push(SSTable::load(path.to_str().unwrap())?);
        }

        thread::sleep(Duration::from_secs(60));

        Ok(())
    }

    pub fn execute(&mut self, cmd: Command) -> Result<Option<String>> {
        match cmd {
            Command::Get(key) => {
                if self.ttl.is_expired(&key) {
                    self.storage.del(&key)?; // Remove from storage if expired
                    self.ttl.remove(&key); // Remove from TTL table
                    return Ok(None); // Return None since the value expired
                }

                match self.storage.get(&key)? {
                    Some(Value::String(val)) => Ok(Some(val)),
                    Some(_) => Ok(None),
                    None => {
                        // If not found in MemTable, check SSTables
                        for sst in &self.sstables {
                            if let Some(Value::String(val)) = sst.get(&key) {
                                return Ok(Some(val));
                            }
                        }
                        Ok(None)
                    }
                }
            }

            Command::Set(key, value) => {
                self.wal.append(LogEntry::Set(key.clone(), value.clone()))?;
                self.storage.set(key, Value::String(value))?;

                if self.storage.len() >= self.flush_threshold {
                    let timestamp = chrono::Utc::now().timestamp();
                    let path = self.sst_dir.join(format!("{}.sst", timestamp));
                    self.storage.flush_to_sstable(path.to_str().unwrap())?;

                    // Load flushed SSTable into memory
                    let new_sst = SSTable::load(path.to_str().unwrap())?;
                    self.sstables.push(new_sst);

                    // Clear MemTable after flushing
                    self.storage.clear();
                }

                Ok(None)
            }

            Command::Del(key) => {
                self.wal.append(LogEntry::Del(key.clone()))?;
                let _existed = self.storage.del(&key)?;
                self.ttl.remove(&key);
                Ok(None)
            }

            // HSet command
            Command::HSet(key, field, value) => {
                let mut map = match self.storage.get(&key)? {
                    Some(Value::Hash(map)) => map,
                    _ => HashMap::new(),
                };

                map.insert(field, value);
                self.storage.set(key.clone(), Value::Hash(map.clone()))?;
                self.wal.append(LogEntry::Set(
                    key,
                    serde_json::to_string(&Value::Hash(map))?,
                ))?;

                Ok(None)
            }

            Command::HGet(key, field) => match self.storage.get(&key)? {
                Some(Value::Hash(map)) => Ok(map.get(&field).cloned()),
                Some(Value::String(_)) => Err(VaporDBError::TypeMismatch(
                    "Expected hash, found string".into(),
                )),
                Some(Value::List(_)) => Err(VaporDBError::TypeMismatch(
                    "Expected hash, found list".into(),
                )),
                Some(Value::Set(_)) => Err(VaporDBError::TypeMismatch(
                    "Expected hash, found set".into(),
                )),
                None => Ok(None),
            },

            // HDel command
            Command::HDel(key, field) => {
                match self.storage.get(&key)? {
                    Some(Value::Hash(mut map)) => {
                        let _removed = map.remove(&field);

                        if map.is_empty() {
                            self.storage.del(&key)?;
                            self.wal.append(LogEntry::Del(key))?;
                        } else {
                            self.storage.set(key.clone(), Value::Hash(map.clone()))?;
                            self.wal.append(LogEntry::Set(
                                key,
                                serde_json::to_string(&Value::Hash(map))?,
                            ))?;
                        }
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected hash, found string".into(),
                        ))
                    }
                    Some(Value::List(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected hash, found list".into(),
                        ))
                    }
                    Some(Value::Set(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected hash, found set".into(),
                        ))
                    }
                    None => {}
                }

                Ok(None)
            }

            // LPush command
            Command::LPush(key, value) => {
                let mut list = match self.storage.get(&key)? {
                    Some(Value::List(list)) => list,
                    _ => Vec::new(),
                };

                list.insert(0, value);
                self.storage.set(key, Value::List(list))?;
                Ok(None)
            }

            // RPush command
            Command::RPush(key, value) => {
                let mut list = match self.storage.get(&key)? {
                    Some(Value::List(list)) => list,
                    _ => Vec::new(),
                };

                list.push(value);
                self.storage.set(key, Value::List(list))?;
                Ok(None)
            }

            // LPop command - Fixed: should remove from beginning, not end
            Command::LPop(key) => {
                match self.storage.get(&key)? {
                    Some(Value::List(mut list)) => {
                        if !list.is_empty() {
                            let value = list.remove(0); // Remove from beginning for LPop
                            if list.is_empty() {
                                self.storage.del(&key)?;
                            } else {
                                self.storage.set(key, Value::List(list))?;
                            }
                            return Ok(Some(value));
                        }
                        Ok(None) // Empty list
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found string".into(),
                        ));
                    }
                    Some(Value::Hash(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found hash".into(),
                        ));
                    }
                    Some(Value::Set(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found set".into(),
                        ));
                    }
                    None => Ok(None), // No such key
                }
            }

            // RPop command - Fixed: should remove from end
            Command::RPop(key) => {
                match self.storage.get(&key)? {
                    Some(Value::List(mut list)) => {
                        if let Some(value) = list.pop() {
                            // Remove from end for RPop
                            if list.is_empty() {
                                self.storage.del(&key)?;
                            } else {
                                self.storage.set(key, Value::List(list))?;
                            }
                            return Ok(Some(value));
                        }
                        Ok(None) // Empty list
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found string".into(),
                        ));
                    }
                    Some(Value::Hash(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found hash".into(),
                        ));
                    }
                    Some(Value::Set(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found set".into(),
                        ));
                    }
                    None => Ok(None), // No such key
                }
            }

            // LRange command - Fixed: proper range handling
            Command::LRange(key, start, end) => {
                match self.storage.get(&key)? {
                    Some(Value::List(list)) => {
                        let len = list.len();
                        if len == 0 {
                            return Ok(Some("[]".to_string()));
                        }

                        // Handle negative indices and bounds
                        let start_idx = start as usize;
                        let end_idx = if end as usize >= len {
                            len - 1
                        } else {
                            end as usize
                        };

                        if start_idx > end_idx || start_idx >= len {
                            return Ok(Some("[]".to_string()));
                        }

                        let range: Vec<String> = list[start_idx..=end_idx].to_vec();
                        Ok(Some(serde_json::to_string(&range)?))
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found string".into(),
                        ));
                    }
                    Some(Value::Hash(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found hash".into(),
                        ));
                    }
                    Some(Value::Set(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected list, found set".into(),
                        ));
                    }
                    None => Ok(Some("[]".to_string())), // No such key
                }
            }

            // SAdd command
            Command::SAdd(key, value) => {
                let mut set = match self.storage.get(&key)? {
                    Some(Value::Set(set)) => set,
                    _ => HashSet::new(),
                };

                set.insert(value);
                self.storage.set(key, Value::Set(set))?;
                Ok(None)
            }

            // SRem command
            Command::SRem(key, value) => {
                match self.storage.get(&key)? {
                    Some(Value::Set(mut set)) => {
                        set.remove(&value); // Remove member from the set
                        if set.is_empty() {
                            self.storage.del(&key)?;
                        } else {
                            self.storage.set(key, Value::Set(set))?;
                        }
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found string".into(),
                        ));
                    }
                    Some(Value::Hash(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found hash".into(),
                        ));
                    }
                    Some(Value::List(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found list".into(),
                        ));
                    }
                    None => {} // Set doesn't exist
                }
                Ok(None)
            }

            // SMembers command
            Command::SMembers(key) => {
                match self.storage.get(&key)? {
                    Some(Value::Set(set)) => {
                        let members: Vec<String> = set.into_iter().collect();
                        Ok(Some(serde_json::to_string(&members)?))
                    }
                    Some(Value::String(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found string".into(),
                        ));
                    }
                    Some(Value::Hash(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found hash".into(),
                        ));
                    }
                    Some(Value::List(_)) => {
                        return Err(VaporDBError::TypeMismatch(
                            "Expected set, found list".into(),
                        ));
                    }
                    None => Ok(None), // No set found
                }
            } // Removed the unreachable pattern catch-all
        }
    }

    pub fn set_with_expiration(&mut self, key: String, value: String, ttl_secs: u64) -> Result<()> {
        self.execute(Command::Set(key.clone(), value))?;
        self.ttl.set(key, Duration::from_secs(ttl_secs));
        Ok(())
    }
}
