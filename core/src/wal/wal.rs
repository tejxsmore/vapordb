use crate::error::{VaporDBError, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug)]
pub enum LogEntry {
    Set(String, String),
    Del(String),
}

pub struct WriteAheadLog {
    path: PathBuf,
    writer: BufWriter<File>,
}

impl WriteAheadLog {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|e| VaporDBError::Internal(e.to_string()))?;

        Ok(Self {
            path,
            writer: BufWriter::new(file),
        })
    }

    pub fn append(&mut self, entry: LogEntry) -> Result<()> {
        let encoded =
            bincode::serialize(&entry).map_err(|e| VaporDBError::Internal(e.to_string()))?;

        self.writer
            .write_all(&(encoded.len() as u32).to_le_bytes())
            .map_err(|e| VaporDBError::Internal(e.to_string()))?;

        self.writer
            .write_all(&encoded)
            .map_err(|e| VaporDBError::Internal(e.to_string()))?;

        self.writer
            .flush()
            .map_err(|e| VaporDBError::Internal(e.to_string()))?;

        Ok(())
    }

    pub fn load_entries(&self) -> Result<Vec<LogEntry>> {
        let mut entries = Vec::new();

        let file = File::open(&self.path).map_err(|e| VaporDBError::Internal(e.to_string()))?;
        let mut reader = BufReader::new(file);
        let mut len_buf = [0u8; 4];

        while reader.read_exact(&mut len_buf).is_ok() {
            let len = u32::from_le_bytes(len_buf) as usize;
            let mut data = vec![0u8; len];
            reader
                .read_exact(&mut data)
                .map_err(|e| VaporDBError::Internal(e.to_string()))?;
            let entry: LogEntry =
                bincode::deserialize(&data).map_err(|e| VaporDBError::Internal(e.to_string()))?;
            entries.push(entry);
        }

        Ok(entries)
    }
}
