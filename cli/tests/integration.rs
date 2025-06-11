use assert_cmd::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_vapordb_cli_commands() {
    // Launch server in a thread
    thread::spawn(|| {
        let mut cmd = Command::cargo_bin("cli").unwrap();
        cmd.arg("start").unwrap();
    });

    // Give the server a second to boot
    std::thread::sleep(Duration::from_secs(1));

    // Set a key
    let mut set = Command::cargo_bin("cli").unwrap();
    set.args(["set", "testkey", "value123"]).assert().success();

    // Get the key
    let mut get = Command::cargo_bin("cli").unwrap();
    get.args(["get", "testkey"])
        .assert()
        .stdout(predicates::str::contains("value123"));

    // Delete the key
    let mut del = Command::cargo_bin("cli").unwrap();
    del.args(["del", "testkey"]).assert().success();

    // Get again: should be empty or silent
    let mut get_again = Command::cargo_bin("cli").unwrap();
    get_again.args(["get", "testkey"]).assert().stdout("");
}

#[test]
fn test_expiration() {
    // Set with expiration
    let mut set = Command::cargo_bin("cli").unwrap();
    set.args(["set-expiring", "temp", "abc", "--ttl", "2"])
        .assert()
        .success();

    let mut get = Command::cargo_bin("cli").unwrap();
    get.args(["get", "temp"])
        .assert()
        .stdout(predicates::str::contains("abc"));

    // Sleep and check again
    std::thread::sleep(Duration::from_secs(3));

    let mut get_after = Command::cargo_bin("cli").unwrap();
    get_after.args(["get", "temp"]).assert().stdout("");
}
