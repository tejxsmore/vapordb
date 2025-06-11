use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::storage::{memtable::MemTable};
use crate::ttl::ExpirationTable;

pub fn start_ttl_daemon(
    expirations: Arc<ExpirationTable>,
    memtable: Arc<MemTable>,
    interval: Duration,
    logging: bool,
) {
    thread::spawn(move || loop {
        thread::sleep(interval);

        let now = std::time::Instant::now();
        let mut expired_keys = vec![];

        {
            let expirations_read = expirations.expirations.read();
            for (key, &when) in expirations_read.iter() {
                if now >= when {
                    expired_keys.push(key.clone());
                }
            }
        }

        if !expired_keys.is_empty() {
            let mut exp_write = expirations.expirations.write();
            let mut mem_write = memtable.map.write().expect("Failed to lock memtable for writing");

            for key in &expired_keys {
                exp_write.remove(key);
                mem_write.remove(key);
                if logging {
                    println!("[TTL] Expired key: {}", key);
                }
            }
}
        if logging {
            println!("[TTL] Checked for expired keys, found: {}", expired_keys.len());
        }
    });
}
