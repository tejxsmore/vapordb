pub mod memtable;
pub mod sst;
pub mod value;

use crate::error::Result;
pub use value::Value;

pub trait Storage: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Value>>;
    fn set(&self, key: String, value: Value) -> Result<()>;
    fn del(&self, key: &str) -> Result<()>;
    fn exists(&self, key: &str) -> Result<bool>;
    fn keys(&self) -> Result<Vec<String>>;    
}
