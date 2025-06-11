use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Hash(HashMap<String, String>),
    List(Vec<String>),
    Set(HashSet<String>),
}
