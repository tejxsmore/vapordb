use core::command::Command;
use core::db::VaporDB;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn setup_db() -> Arc<Mutex<VaporDB>> {
    let db = VaporDB::new_with_persistence("test_adv.wal").unwrap();
    Arc::new(Mutex::new(db))
}

#[test]
fn test_list_lrange_bounds() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::RPush("list".into(), "1".into())).unwrap();
    db.execute(Command::RPush("list".into(), "2".into())).unwrap();
    db.execute(Command::RPush("list".into(), "3".into())).unwrap();

    let range = db.execute(Command::LRange("list".into(), 0, 1)).unwrap().unwrap();
    let parsed: Vec<String> = serde_json::from_str(&range).unwrap();
    assert_eq!(parsed, vec!["1", "2"]);

    let full = db.execute(Command::LRange("list".into(), 0, usize::MAX)).unwrap().unwrap();
    let parsed: Vec<String> = serde_json::from_str(&full).unwrap();
    assert_eq!(parsed, vec!["1", "2", "3"]);
}

#[test]
fn test_lrange_empty_and_out_of_bounds() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    let range = db.execute(Command::LRange("nosuch".into(), 0, 5)).unwrap();
    assert_eq!(range, Some("[]".into()));

    db.execute(Command::LPush("mylist".into(), "val".into())).unwrap();
    let out = db.execute(Command::LRange("mylist".into(), 5, 10)).unwrap();
    let parsed: Vec<String> = serde_json::from_str(&out.unwrap()).unwrap();
    assert!(parsed.is_empty());
}

#[test]
fn test_ttl_update_behavior() {
    let db = setup_db();
    VaporDB::start_ttl_daemon(db.clone());

    {
        let mut db = db.lock().unwrap();
        db.set_with_expiration("expire".into(), "short".into(), 1).unwrap();
        thread::sleep(Duration::from_millis(500));
        db.set_with_expiration("expire".into(), "long".into(), 2).unwrap();
    }

    thread::sleep(Duration::from_secs(1));

    {
        let mut db = db.lock().unwrap();
        let val = db.execute(Command::Get("expire".into())).unwrap();
        assert_eq!(val, Some("long".into()));
    }

    thread::sleep(Duration::from_secs(2));
    let mut db = db.lock().unwrap();
    assert_eq!(db.execute(Command::Get("expire".into())).unwrap(), None);
}

#[test]
fn test_concurrent_set_get_integrity() {
    let db = setup_db();
    let db_arc1 = db.clone();
    let db_arc2 = db.clone();

    let t1 = thread::spawn(move || {
        for i in 0..100 {
            let mut db = db_arc1.lock().unwrap();
            db.execute(Command::Set(format!("key{i}"), format!("val{i}"))).unwrap();
        }
    });

    let t2 = thread::spawn(move || {
        for i in 0..100 {
            let mut db = db_arc2.lock().unwrap();
            db.execute(Command::Set(format!("key_alt{i}"), format!("val_alt{i}"))).unwrap();
        }
    });

    t1.join().unwrap();
    t2.join().unwrap();

    let mut db = db.lock().unwrap();
    for i in 0..100 {
        let v = db.execute(Command::Get(format!("key{i}"))).unwrap();
        assert_eq!(v, Some(format!("val{i}")));
        let v = db.execute(Command::Get(format!("key_alt{i}"))).unwrap();
        assert_eq!(v, Some(format!("val_alt{i}")));
    }
}

#[test]
fn test_list_behavior_and_range() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::LPush("list".into(), "a".into())).unwrap();
    db.execute(Command::RPush("list".into(), "b".into())).unwrap();
    db.execute(Command::RPush("list".into(), "c".into())).unwrap();

    let res = db.execute(Command::LRange("list".into(), 0, 10)).unwrap().unwrap();
    let vals: Vec<String> = serde_json::from_str(&res).unwrap();
    assert_eq!(vals, vec!["a", "b", "c"]);

    let res = db.execute(Command::LRange("list".into(), 10, 20)).unwrap().unwrap();
    let vals: Vec<String> = serde_json::from_str(&res).unwrap();
    assert!(vals.is_empty());
}

#[test]
fn test_hash_multiple_fields() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::HSet("multi".into(), "f1".into(), "v1".into())).unwrap();
    db.execute(Command::HSet("multi".into(), "f2".into(), "v2".into())).unwrap();
    db.execute(Command::HSet("multi".into(), "f3".into(), "v3".into())).unwrap();

    let val1 = db.execute(Command::HGet("multi".into(), "f1".into())).unwrap();
    let val2 = db.execute(Command::HGet("multi".into(), "f2".into())).unwrap();
    let val3 = db.execute(Command::HGet("multi".into(), "f3".into())).unwrap();

    assert_eq!(val1, Some("v1".into()));
    assert_eq!(val2, Some("v2".into()));
    assert_eq!(val3, Some("v3".into()));
}

#[test]
fn test_set_uniqueness_and_membership() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::SAdd("set".into(), "a".into())).unwrap();
    db.execute(Command::SAdd("set".into(), "b".into())).unwrap();
    db.execute(Command::SAdd("set".into(), "a".into())).unwrap(); // duplicate

    let members = db.execute(Command::SMembers("set".into())).unwrap().unwrap();
    let parsed: Vec<String> = serde_json::from_str(&members).unwrap();

    assert_eq!(parsed.len(), 2);
    assert!(parsed.contains(&"a".to_string()));
    assert!(parsed.contains(&"b".to_string()));

    db.execute(Command::SRem("set".into(), "a".into())).unwrap();
    let members = db.execute(Command::SMembers("set".into())).unwrap().unwrap();
    let parsed: Vec<String> = serde_json::from_str(&members).unwrap();

    assert_eq!(parsed, vec!["b"]);
}

#[test]
fn test_string_ttl_expiration_hard() {
    let db = setup_db();
    VaporDB::start_ttl_daemon(db.clone());

    {
        let mut db = db.lock().unwrap();
        db.set_with_expiration("ttlkey".into(), "will_expire".into(), 1).unwrap();
        assert_eq!(db.execute(Command::Get("ttlkey".into())).unwrap(), Some("will_expire".into()));
    }

    thread::sleep(Duration::from_secs(2));

    {
        let mut db = db.lock().unwrap();
        let val = db.execute(Command::Get("ttlkey".into())).unwrap();
        assert_eq!(val, None);
    }
}

#[test]
fn test_set_removal_of_nonexistent_element() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    db.execute(Command::SAdd("settest".into(), "x".into())).unwrap();
    db.execute(Command::SRem("settest".into(), "y".into())).unwrap();

    let members = db.execute(Command::SMembers("settest".into())).unwrap().unwrap();
    let parsed: Vec<String> = serde_json::from_str(&members).unwrap();

    assert_eq!(parsed, vec!["x"]);
}

#[test]
fn test_list_pop_from_empty() {
    let db = setup_db();
    let mut db = db.lock().unwrap();

    let left = db.execute(Command::LPop("emptylist".into())).unwrap();
    let right = db.execute(Command::RPop("emptylist".into())).unwrap();

    assert_eq!(left, None);
    assert_eq!(right, None);
}

#[test]
fn test_ttl_reset_after_expired() {
    let db = setup_db();
    VaporDB::start_ttl_daemon(db.clone());

    {
        let mut db = db.lock().unwrap();
        db.set_with_expiration("resettl".into(), "one".into(), 1).unwrap();
    }

    thread::sleep(Duration::from_secs(2));

    {
        let mut db = db.lock().unwrap();
        assert_eq!(db.execute(Command::Get("resettl".into())).unwrap(), None);
        db.set_with_expiration("resettl".into(), "two".into(), 2).unwrap();
    }

    thread::sleep(Duration::from_secs(1));

    {
        let mut db = db.lock().unwrap();
        assert_eq!(db.execute(Command::Get("resettl".into())).unwrap(), Some("two".into()));
    }
}
