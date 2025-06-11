#[derive(Debug)]
pub enum Command {
    Get(String),
    Set(String, String),
    Del(String),

    // Hash operations
    HSet(String, String, String),
    HGet(String, String),
    HDel(String, String),

    // ✅ List commands
    LPush(String, String),
    RPush(String, String),
    LPop(String),
    RPop(String),
    LRange(String, usize, usize),

    // ✅ Set commands
    SAdd(String, String),
    SRem(String, String),
    SMembers(String),
}
