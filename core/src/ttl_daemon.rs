use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::storage::Value;
use crate::storage::memtable::MemTable;
use crate::ttl::ExpirationTable;
use crate::storage::sst::SSTable;

const SSTABLE_PATH: &str = "sstable.json";

pub fn start_ttl_daemon(
    expirations: Arc<ExpirationTable>,
    memtable: Arc<MemTable>,
    sstable: Option<Arc<Mutex<SSTable>>>,
    interval: Duration,
    logging: bool,
) {
    thread::Builder::new()
        .name("ttl_daemon".into())
        .spawn(move || loop {
            thread::sleep(interval);

            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("System time error")
                .as_secs();

            let mut expired_keys = Vec::new();

            // Step 1: Identify expired keys
            {
                let expirations_read = expirations.expirations.read();
                for (key, &expire_at) in expirations_read.iter() {
                    if now >= expire_at {
                        expired_keys.push(key.clone());
                    }
                }
            }

            if expired_keys.is_empty() {
                if logging {
                    println!("[TTL] Checked for expired keys, none found.");
                }
                continue;
            }

            // Step 2: Remove from expiration table and memtable
            let mut exp_write = expirations.expirations.write();
            let mut mem_write = match memtable.map.write() {
                Ok(guard) => guard,
                Err(poisoned) => {
                    eprintln!("[TTL] MemTable write lock poisoned, continuing with inner state.");
                    poisoned.into_inner()
                }
            };

            for key in &expired_keys {
                exp_write.remove(key);
                mem_write.remove(key);
            }

            // Step 3: Try to update SSTable if present
            let mut sstable_map: HashMap<String, Option<Value>> = mem_write
                .iter()
                .map(|(k, v)| (k.clone(), Some(v.clone())))
                .collect();

            if let Some(sstable) = &sstable {
                match sstable.lock() {
                    Ok(mut sstable_guard) => {
                        for key in &expired_keys {
                            sstable_guard.delete(key); // mark as tombstone
                        }

                        // Include tombstones (overwrite values from memtable if necessary)
                        for (k, v) in sstable_guard.map.iter() {
                            if v.is_none() {
                                sstable_map.insert(k.clone(), None);
                            }
                        }

                        let ttl_map: HashMap<String, u64> = exp_write
                            .iter()
                            .map(|(k, &v)| (k.clone(), v))
                            .collect();

                        if let Err(e) = SSTable::write(SSTABLE_PATH, &sstable_map, &ttl_map) {
                            eprintln!("[TTL] Failed to write SSTable: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("[TTL] Failed to lock SSTable: {}", e);
                    }
                }
            } else if logging {
                println!("[TTL] SSTable is None, skipping disk cleanup.");
            }

            if logging {
                println!("[TTL] Expired keys removed: {:?}", expired_keys);
            }
        })
        .expect("Failed to start TTL daemon thread");
}
