use core::db::VaporDB;
use core::command::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn setup_db() -> Arc<Mutex<VaporDB>> {
    let db = VaporDB::new_with_persistence("test.wal").unwrap();
    Arc::new(Mutex::new(db))
}

#[test]
fn test_string_set_get() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::Set("k".into(), "v".into())).unwrap();
    let result = db.execute(Command::Get("k".into())).unwrap();
    assert_eq!(result, Some("v".to_string()));
}

#[test]
fn test_list_push_pop() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::LPush("mylist".into(), "a".into())).unwrap();
    db.execute(Command::RPush("mylist".into(), "b".into())).unwrap();
    db.execute(Command::LPush("mylist".into(), "c".into())).unwrap();

    let val = db.execute(Command::LPop("mylist".into())).unwrap();
    assert_eq!(val, Some("c".into()));

    let val = db.execute(Command::RPop("mylist".into())).unwrap();
    assert_eq!(val, Some("b".into()));

    let val = db.execute(Command::LPop("mylist".into())).unwrap();
    assert_eq!(val, Some("a".into()));

    let val = db.execute(Command::LPop("mylist".into())).unwrap();
    assert_eq!(val, None);
}

#[test]
fn test_set_operations() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::SAdd("myset".into(), "x".into())).unwrap();
    db.execute(Command::SAdd("myset".into(), "x".into())).unwrap(); // duplicate
    db.execute(Command::SAdd("myset".into(), "y".into())).unwrap();

    let members = db.execute(Command::SMembers("myset".into())).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&members.unwrap()).unwrap();
    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains(&"x".to_string()));
    assert!(parsed.contains(&"y".to_string()));

    db.execute(Command::SRem("myset".into(), "x".into())).unwrap();
    let members = db.execute(Command::SMembers("myset".into())).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&members.unwrap()).unwrap();
    assert_eq!(parsed, vec!["y"]);
}

#[test]
fn test_hash_hset_hget_hdel() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::HSet("myhash".into(), "f1".into(), "v1".into())).unwrap();
    db.execute(Command::HSet("myhash".into(), "f2".into(), "v2".into())).unwrap();

    let v1 = db.execute(Command::HGet("myhash".into(), "f1".into())).unwrap();
    assert_eq!(v1, Some("v1".into()));

    db.execute(Command::HDel("myhash".into(), "f1".into())).unwrap();
    let v1 = db.execute(Command::HGet("myhash".into(), "f1".into())).unwrap();
    assert_eq!(v1, None);
}

#[test]
fn test_ttl_expiration() {
    let db = setup_db();
    let db_arc = Arc::clone(&db);
    VaporDB::start_ttl_daemon(Arc::clone(&db_arc));

    {
        let mut db = db.lock().unwrap();
        db.set_with_expiration("temp".into(), "bye".into(), 1).unwrap();
        assert_eq!(db.execute(Command::Get("temp".into())).unwrap(), Some("bye".into()));
    }

    thread::sleep(Duration::from_secs(2));

    let mut db = db.lock().unwrap();
    let val = db.execute(Command::Get("temp".into())).unwrap();
    assert_eq!(val, None);
}
