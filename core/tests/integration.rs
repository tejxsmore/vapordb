use core::command::Command;
use core::db::VaporDB;
use std::fs;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn test_basic_string_operations() {
    let wal_path = "vapordb_test_strings.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing basic string operations...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // SET and GET
    db.execute(Command::Set("key1".into(), "value1".into()))
        .unwrap();
    let val = db.execute(Command::Get("key1".into())).unwrap();
    assert_eq!(val, Some("value1".into()));
    println!("✓ SET/GET works");

    // Overwrite existing key
    db.execute(Command::Set("key1".into(), "new_value".into()))
        .unwrap();
    let val = db.execute(Command::Get("key1".into())).unwrap();
    assert_eq!(val, Some("new_value".into()));
    println!("✓ Key overwrite works");

    // DEL
    db.execute(Command::Del("key1".into())).unwrap();
    let val = db.execute(Command::Get("key1".into())).unwrap();
    assert_eq!(val, None);
    println!("✓ DEL works");

    // GET non-existent key
    let val = db.execute(Command::Get("nonexistent".into())).unwrap();
    assert_eq!(val, None);
    println!("✓ Non-existent key returns None");

    let _ = fs::remove_file(wal_path);
    println!("✅ Basic string operations test passed\n");
}

#[test]
fn test_hash_operations() {
    let wal_path = "vapordb_test_hashes.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing hash operations...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // HSET and HGET
    db.execute(Command::HSet(
        "user:1".into(),
        "name".into(),
        "Alice".into(),
    ))
    .unwrap();
    db.execute(Command::HSet("user:1".into(), "age".into(), "30".into()))
        .unwrap();
    db.execute(Command::HSet("user:1".into(), "city".into(), "NYC".into()))
        .unwrap();

    let name = db
        .execute(Command::HGet("user:1".into(), "name".into()))
        .unwrap();
    assert_eq!(name, Some("Alice".into()));
    println!("✓ HSET/HGET works");

    let age = db
        .execute(Command::HGet("user:1".into(), "age".into()))
        .unwrap();
    assert_eq!(age, Some("30".into()));

    // Update existing hash field
    db.execute(Command::HSet("user:1".into(), "age".into(), "31".into()))
        .unwrap();
    let updated_age = db
        .execute(Command::HGet("user:1".into(), "age".into()))
        .unwrap();
    assert_eq!(updated_age, Some("31".into()));
    println!("✓ Hash field update works");

    // HDEL
    db.execute(Command::HDel("user:1".into(), "age".into()))
        .unwrap();
    let missing_age = db
        .execute(Command::HGet("user:1".into(), "age".into()))
        .unwrap();
    assert_eq!(missing_age, None);
    println!("✓ HDEL works");

    // Verify other fields still exist
    let name = db
        .execute(Command::HGet("user:1".into(), "name".into()))
        .unwrap();
    assert_eq!(name, Some("Alice".into()));
    println!("✓ Other hash fields preserved");

    // HGET from non-existent hash
    let missing = db
        .execute(Command::HGet("nonexistent".into(), "field".into()))
        .unwrap();
    assert_eq!(missing, None);
    println!("✓ Non-existent hash returns None");

    let _ = fs::remove_file(wal_path);
    println!("✅ Hash operations test passed\n");
}

#[test]
fn test_list_operations() {
    let wal_path = "vapordb_test_lists.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing list operations...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // LPUSH and RPUSH
    db.execute(Command::LPush("mylist".into(), "first".into()))
        .unwrap();
    db.execute(Command::RPush("mylist".into(), "last".into()))
        .unwrap();
    db.execute(Command::LPush("mylist".into(), "beginning".into()))
        .unwrap();
    db.execute(Command::RPush("mylist".into(), "end".into()))
        .unwrap();
    println!("✓ LPUSH/RPUSH works");

    // List should now be: [beginning, first, last, end]

    // LRANGE - get all elements
    let all = db.execute(Command::LRange("mylist".into(), 0, 10)).unwrap();
    if let Some(list_json) = all {
        // Parse the JSON array returned by LRange
        let elements: Vec<String> = serde_json::from_str(&list_json).unwrap();
        assert_eq!(elements.len(), 4);
        assert_eq!(elements[0], "beginning");
        assert_eq!(elements[1], "first");
        assert_eq!(elements[2], "last");
        assert_eq!(elements[3], "end");
        println!("✓ LRANGE works - got {} elements", elements.len());
    } else {
        panic!("Expected list data");
    }

    // LRANGE - partial range
    let partial = db.execute(Command::LRange("mylist".into(), 1, 2)).unwrap();
    if let Some(list_json) = partial {
        let elements: Vec<String> = serde_json::from_str(&list_json).unwrap();
        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0], "first");
        assert_eq!(elements[1], "last");
        println!("✓ LRANGE partial works");
    } else {
        panic!("Expected partial list data");
    }

    // LPOP and RPOP
    let left_popped = db.execute(Command::LPop("mylist".into())).unwrap();
    assert_eq!(left_popped, Some("beginning".into()));
    println!("✓ LPOP works");

    let right_popped = db.execute(Command::RPop("mylist".into())).unwrap();
    assert_eq!(right_popped, Some("end".into()));
    println!("✓ RPOP works");

    // Pop from empty list after clearing
    db.execute(Command::LPop("mylist".into())).unwrap();
    db.execute(Command::LPop("mylist".into())).unwrap();

    let empty_pop = db.execute(Command::LPop("mylist".into())).unwrap();
    assert_eq!(empty_pop, None);
    println!("✓ Empty list pop returns None");

    let _ = fs::remove_file(wal_path);
    println!("✅ List operations test passed\n");
}

#[test]
fn test_set_operations() {
    let wal_path = "vapordb_test_sets.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing set operations...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // SADD - add members to set
    db.execute(Command::SAdd("myset".into(), "apple".into()))
        .unwrap();
    db.execute(Command::SAdd("myset".into(), "banana".into()))
        .unwrap();
    db.execute(Command::SAdd("myset".into(), "cherry".into()))
        .unwrap();
    println!("✓ SADD works");

    // Add duplicate (should be ignored)
    db.execute(Command::SAdd("myset".into(), "apple".into()))
        .unwrap();

    // SMEMBERS - get all members
    let members = db.execute(Command::SMembers("myset".into())).unwrap();
    if let Some(members_json) = members {
        let member_vec: Vec<String> = serde_json::from_str(&members_json).unwrap();
        assert_eq!(member_vec.len(), 3);
        // Note: HashSet order is not guaranteed, so we check contains
        assert!(member_vec.contains(&"apple".to_string()));
        assert!(member_vec.contains(&"banana".to_string()));
        assert!(member_vec.contains(&"cherry".to_string()));
        println!("✓ SMEMBERS works - got {} members", member_vec.len());
    } else {
        panic!("Expected set members");
    }

    // SREM - remove member
    db.execute(Command::SRem("myset".into(), "banana".into()))
        .unwrap();

    let members_after_rem = db.execute(Command::SMembers("myset".into())).unwrap();
    if let Some(members_json) = members_after_rem {
        let member_vec: Vec<String> = serde_json::from_str(&members_json).unwrap();
        assert_eq!(member_vec.len(), 2);
        assert!(member_vec.contains(&"apple".to_string()));
        assert!(member_vec.contains(&"cherry".to_string()));
        assert!(!member_vec.contains(&"banana".to_string()));
        println!("✓ SREM works");
    }

    // SMEMBERS on non-existent set
    let nonexistent = db
        .execute(Command::SMembers("nonexistent_set".into()))
        .unwrap();
    assert_eq!(nonexistent, None);
    println!("✓ Non-existent set returns None");

    let _ = fs::remove_file(wal_path);
    println!("✅ Set operations test passed\n");
}

#[test]
fn test_data_type_conflicts() {
    let wal_path = "vapordb_test_conflicts.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing data type conflicts...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    let key = "data".to_string();

    // Start with string
    db.execute(Command::Set(key.clone(), "string_value".into()))
        .unwrap();
    let val = db.execute(Command::Get(key.clone())).unwrap();
    assert_eq!(val, Some("string_value".into()));
    println!("✓ String value set");

    // Try to use same key as hash - should work (overwrite)
    db.execute(Command::HSet(
        key.clone(),
        "field1".into(),
        "hash_value".into(),
    ))
    .unwrap();
    let hash_val = db
        .execute(Command::HGet(key.clone(), "field1".into()))
        .unwrap();
    assert_eq!(hash_val, Some("hash_value".into()));
    println!("✓ Hash overwrites string");

    // The string should be gone now
    let string_val = db.execute(Command::Get(key.clone())).unwrap();
    assert_eq!(string_val, None);
    println!("✓ Original string value removed");

    let _ = fs::remove_file(wal_path);
    println!("✅ Data type conflicts test passed\n");
}

#[test]
fn test_expiration() {
    let wal_path = "vapordb_test_expiration.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing expiration...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // Set value with expiration
    db.set_with_expiration("temp_key".into(), "expires_soon".into(), 1)
        .unwrap();

    // Should exist immediately
    let val = db.execute(Command::Get("temp_key".into())).unwrap();
    assert_eq!(val, Some("expires_soon".into()));
    println!("✓ Value exists before expiration");

    // Wait for expiration
    sleep(Duration::from_secs(2));

    // Should be expired now
    let expired_val = db.execute(Command::Get("temp_key".into())).unwrap();
    assert_eq!(expired_val, None);
    println!("✓ Value expired after TTL");

    // Test that regular values don't expire
    db.execute(Command::Set("permanent".into(), "stays".into()))
        .unwrap();
    sleep(Duration::from_secs(1));
    let permanent_val = db.execute(Command::Get("permanent".into())).unwrap();
    assert_eq!(permanent_val, Some("stays".into()));
    println!("✓ Non-expiring values remain");

    let _ = fs::remove_file(wal_path);
    println!("✅ Expiration test passed\n");
}

#[test]
fn test_edge_cases() {
    let wal_path = "vapordb_test_edge_cases.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing edge cases...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // Empty string values
    db.execute(Command::Set("empty".into(), "".into())).unwrap();
    assert_eq!(
        db.execute(Command::Get("empty".into())).unwrap(),
        Some("".into())
    );
    println!("✓ Empty string values work");

    // Special characters in keys and values
    db.execute(Command::Set(
        "key with spaces".into(),
        "value with spaces".into(),
    ))
    .unwrap();
    assert_eq!(
        db.execute(Command::Get("key with spaces".into())).unwrap(),
        Some("value with spaces".into())
    );
    println!("✓ Keys/values with spaces work");

    // Unicode
    db.execute(Command::Set("unicode🔑".into(), "unicode🔥value".into()))
        .unwrap();
    assert_eq!(
        db.execute(Command::Get("unicode🔑".into())).unwrap(),
        Some("unicode🔥value".into())
    );
    println!("✓ Unicode keys/values work");

    let _ = fs::remove_file(wal_path);
    println!("✅ Edge cases test passed\n");
}

#[test]
fn test_comprehensive_workflow() {
    let wal_path = "vapordb_test_workflow.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing comprehensive workflow...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // User registration
    db.execute(Command::HSet(
        "user:1".into(),
        "name".into(),
        "Alice".into(),
    ))
    .unwrap();
    db.execute(Command::HSet(
        "user:1".into(),
        "email".into(),
        "alice@example.com".into(),
    ))
    .unwrap();
    db.execute(Command::HSet("user:1".into(), "age".into(), "25".into()))
        .unwrap();
    println!("✓ User registration complete");

    // User session with expiration
    db.set_with_expiration("session:abc123".into(), "user:1".into(), 2)
        .unwrap();
    println!("✓ Session created with TTL");

    // User's shopping cart (list)
    db.execute(Command::LPush("cart:user:1".into(), "laptop".into()))
        .unwrap();
    db.execute(Command::RPush("cart:user:1".into(), "mouse".into()))
        .unwrap();
    db.execute(Command::RPush("cart:user:1".into(), "keyboard".into()))
        .unwrap();
    println!("✓ Shopping cart created");

    // User's favorite categories (set)
    db.execute(Command::SAdd(
        "favorites:user:1".into(),
        "electronics".into(),
    ))
    .unwrap();
    db.execute(Command::SAdd("favorites:user:1".into(), "books".into()))
        .unwrap();
    db.execute(Command::SAdd("favorites:user:1".into(), "gaming".into()))
        .unwrap();
    println!("✓ Favorites set created");

    // Cache some computed data
    db.execute(Command::Set(
        "cache:user:1:total_orders".into(),
        "42".into(),
    ))
    .unwrap();
    println!("✓ Cache data stored");

    // Verify user data
    assert_eq!(
        db.execute(Command::HGet("user:1".into(), "name".into()))
            .unwrap(),
        Some("Alice".into())
    );
    assert_eq!(
        db.execute(Command::Get("session:abc123".into())).unwrap(),
        Some("user:1".into())
    );

    // Verify cart has items
    let cart = db
        .execute(Command::LRange("cart:user:1".into(), 0, 10))
        .unwrap();
    assert!(cart.is_some());
    println!("✓ All data structures verified");

    // User removes item from cart
    let removed = db.execute(Command::LPop("cart:user:1".into())).unwrap();
    assert_eq!(removed, Some("laptop".into()));
    println!("✓ Item removed from cart");

    // User removes favorite category
    db.execute(Command::SRem("favorites:user:1".into(), "gaming".into()))
        .unwrap();
    println!("✓ Favorite category removed");

    // Wait for session to expire
    println!("⏳ Waiting for session expiration...");
    sleep(Duration::from_secs(3));
    assert_eq!(
        db.execute(Command::Get("session:abc123".into())).unwrap(),
        None
    );
    println!("✓ Session expired");

    // But user data should still exist
    assert_eq!(
        db.execute(Command::HGet("user:1".into(), "name".into()))
            .unwrap(),
        Some("Alice".into())
    );
    println!("✓ User data persists after session expiration");

    let _ = fs::remove_file(wal_path);
    println!("✅ Comprehensive workflow test passed\n");
}

#[test]
fn test_active_ttl_cleanup() {
    let wal_path = "vapordb_test_ttl_cleanup.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing active TTL cleanup...");

    let mut db = VaporDB::new_with_persistence(wal_path).unwrap();

    // Set multiple keys with different expiration times
    db.set_with_expiration("key1".into(), "expires_in_1s".into(), 1).unwrap();
    db.set_with_expiration("key2".into(), "expires_in_2s".into(), 2).unwrap();
    db.set_with_expiration("key3".into(), "expires_in_3s".into(), 3).unwrap();
    db.set_with_expiration("key4".into(), "expires_in_4s".into(), 4).unwrap();
    
    // Add some permanent keys to ensure they're not affected
    db.execute(Command::Set("permanent1".into(), "stays_forever".into())).unwrap();
    db.execute(Command::Set("permanent2".into(), "also_stays".into())).unwrap();
    
    println!("✓ Set up keys with various TTLs and permanent keys");

    // Verify all keys exist initially
    assert_eq!(db.execute(Command::Get("key1".into())).unwrap(), Some("expires_in_1s".into()));
    assert_eq!(db.execute(Command::Get("key2".into())).unwrap(), Some("expires_in_2s".into()));
    assert_eq!(db.execute(Command::Get("key3".into())).unwrap(), Some("expires_in_3s".into()));
    assert_eq!(db.execute(Command::Get("key4".into())).unwrap(), Some("expires_in_4s".into()));
    assert_eq!(db.execute(Command::Get("permanent1".into())).unwrap(), Some("stays_forever".into()));
    assert_eq!(db.execute(Command::Get("permanent2".into())).unwrap(), Some("also_stays".into()));
    println!("✓ All keys exist initially");

    // Wait 1.5 seconds - key1 should be expired
    sleep(Duration::from_millis(1500));
    
    // Trigger cleanup by accessing keys (this should clean up expired ones)
    let _ = db.execute(Command::Get("key1".into())).unwrap(); // Should be None and trigger cleanup
    
    // Verify key1 is gone but others remain
    assert_eq!(db.execute(Command::Get("key1".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key2".into())).unwrap(), Some("expires_in_2s".into()));
    assert_eq!(db.execute(Command::Get("key3".into())).unwrap(), Some("expires_in_3s".into()));
    assert_eq!(db.execute(Command::Get("key4".into())).unwrap(), Some("expires_in_4s".into()));
    println!("✓ key1 expired and cleaned up, others remain");

    // Wait another 1 second (total 2.5s) - key2 should now be expired
    sleep(Duration::from_millis(1000));
    
    // Trigger cleanup again
    let _ = db.execute(Command::Get("key2".into())).unwrap();
    
    assert_eq!(db.execute(Command::Get("key1".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key2".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key3".into())).unwrap(), Some("expires_in_3s".into()));
    assert_eq!(db.execute(Command::Get("key4".into())).unwrap(), Some("expires_in_4s".into()));
    println!("✓ key2 expired and cleaned up, key3 and key4 remain");

    // Wait until all TTL keys should be expired (total 5s)
    sleep(Duration::from_millis(2000));
    
    // Trigger cleanup by accessing each key
    let _ = db.execute(Command::Get("key3".into())).unwrap();
    let _ = db.execute(Command::Get("key4".into())).unwrap();
    
    // All TTL keys should be gone
    assert_eq!(db.execute(Command::Get("key1".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key2".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key3".into())).unwrap(), None);
    assert_eq!(db.execute(Command::Get("key4".into())).unwrap(), None);
    println!("✓ All TTL keys expired and cleaned up");

    // Permanent keys should still exist
    assert_eq!(db.execute(Command::Get("permanent1".into())).unwrap(), Some("stays_forever".into()));
    assert_eq!(db.execute(Command::Get("permanent2".into())).unwrap(), Some("also_stays".into()));
    println!("✓ Permanent keys unaffected by TTL cleanup");

    // Test cleanup with different data types
    db.set_with_expiration("temp_hash_key".into(), "will_be_hash".into(), 1).unwrap();
    db.set_with_expiration("temp_list_key".into(), "will_be_list".into(), 1).unwrap();
    db.set_with_expiration("temp_set_key".into(), "will_be_set".into(), 1).unwrap();

    // Convert to different data types (this should overwrite and remove TTL)
    db.execute(Command::HSet("temp_hash_key".into(), "field".into(), "value".into())).unwrap();
    db.execute(Command::LPush("temp_list_key".into(), "item".into())).unwrap();
    db.execute(Command::SAdd("temp_set_key".into(), "member".into())).unwrap();

    // Wait for original TTL to pass
    sleep(Duration::from_millis(1500));

    // These should still exist because they were converted to different data types
    assert_eq!(db.execute(Command::HGet("temp_hash_key".into(), "field".into())).unwrap(), Some("value".into()));
    
    let list_result = db.execute(Command::LRange("temp_list_key".into(), 0, 10)).unwrap();
    assert!(list_result.is_some());
    
    let set_result = db.execute(Command::SMembers("temp_set_key".into())).unwrap();
    assert!(set_result.is_some());
    
    println!("✓ Data type conversion removes TTL behavior");

    // Test mass expiration cleanup
    for i in 0..100 {
        db.set_with_expiration(format!("bulk_key_{}", i), format!("bulk_value_{}", i), 1).unwrap();
    }
    println!("✓ Created 100 keys with 1s TTL");

    // Wait for all to expire
    sleep(Duration::from_millis(1500));

    // Trigger cleanup by doing any operation
    let _ = db.execute(Command::Get("bulk_key_0".into())).unwrap();

    // Verify they're all cleaned up
    let mut found_any = false;
    for i in 0..100 {
        if db.execute(Command::Get(format!("bulk_key_{}", i))).unwrap().is_some() {
            found_any = true;
            break;
        }
    }
    assert!(!found_any, "All bulk keys should be expired and cleaned up");
    println!("✓ Mass expiration cleanup works");

    let _ = fs::remove_file(wal_path);
    println!("✅ Active TTL cleanup test passed\n");
}

#[test]
fn test_ttl_cleanup_persistence() {
    let wal_path = "vapordb_test_ttl_persistence.wal";
    let _ = fs::remove_file(wal_path);
    println!("Testing TTL cleanup with persistence...");

    // Create database and add expiring keys
    {
        let mut db = VaporDB::new_with_persistence(wal_path).unwrap();
        
        db.set_with_expiration("short_lived".into(), "expires_soon".into(), 1).unwrap();
        db.set_with_expiration("long_lived".into(), "expires_later".into(), 10).unwrap();
        db.execute(Command::Set("permanent".into(), "forever".into())).unwrap();
        
        println!("✓ Created keys with different TTLs");
        
        // Verify they exist
        assert_eq!(db.execute(Command::Get("short_lived".into())).unwrap(), Some("expires_soon".into()));
        assert_eq!(db.execute(Command::Get("long_lived".into())).unwrap(), Some("expires_later".into()));
        assert_eq!(db.execute(Command::Get("permanent".into())).unwrap(), Some("forever".into()));
    } // Database is dropped here, forcing persistence

    // Wait for short-lived key to expire
    sleep(Duration::from_millis(1500));

    // Reload database from persistence
    {
        let mut db = VaporDB::new_with_persistence(wal_path).unwrap();
        println!("✓ Database reloaded from persistence");

        // VaporDB uses lazy/active TTL cleanup, so expired keys are only removed when accessed
        // The key should be considered expired when we try to get it
        let short_lived_result = db.execute(Command::Get("short_lived".into())).unwrap();
        assert_eq!(short_lived_result, None, "short_lived key should be expired and return None");
        
        // Long-lived key should still exist
        assert_eq!(db.execute(Command::Get("long_lived".into())).unwrap(), Some("expires_later".into()));
        
        // Permanent key should always exist
        assert_eq!(db.execute(Command::Get("permanent".into())).unwrap(), Some("forever".into()));
        
        println!("✓ Expired key cleaned up on access, others preserved");
        
        // Verify that accessing the expired key again still returns None
        assert_eq!(db.execute(Command::Get("short_lived".into())).unwrap(), None);
        println!("✓ Subsequent access to expired key still returns None");
        
        // Test that TTL state persists across operations
        // Try some other operations to ensure the cleanup was permanent
        db.execute(Command::Set("test_key".into(), "test_value".into())).unwrap();
        assert_eq!(db.execute(Command::Get("short_lived".into())).unwrap(), None);
        assert_eq!(db.execute(Command::Get("long_lived".into())).unwrap(), Some("expires_later".into()));
        
        println!("✓ TTL cleanup state consistent across multiple operations");
    }

    let _ = fs::remove_file(wal_path);
    println!("✅ TTL cleanup with persistence test passed\n");
}
