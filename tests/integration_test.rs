use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

// helper to create a test env file
fn create_test_env(dir: &TempDir) -> std::path::PathBuf {
    let file_path = dir.path().join("test.env");
    fs::write(
        &file_path,
        "# Test header\n\nFOO=bar\nBAR=baz # has comment\nQUX=value\n",
    )
    .unwrap();
    file_path
}

// helper to create a command for the envq binary
fn envq_cmd() -> assert_cmd::Command {
    assert_cmd::Command::new(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("envq"),
    )
}

// ignored because atty detection doesn't work properly in test environments
// work when testing manually
#[test]
#[ignore]
fn test_get_missing_file_or_stdin() {
    envq_cmd()
        .arg("get")
        .arg("FOO")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing file or stdin"));
}

#[test]
#[ignore]
fn test_list_missing_file_or_stdin() {
    envq_cmd()
        .arg("list")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Missing file or stdin"));
}

#[test]
fn test_get_with_piped_input_found() {
    envq_cmd()
        .arg("get")
        .arg("FOO")
        .write_stdin("FOO=bar\nBAR=baz\n")
        .assert()
        .success()
        .code(0)
        .stdout("bar\n");
}

#[test]
fn test_get_with_piped_input_not_found() {
    envq_cmd()
        .arg("get")
        .arg("NOTFOUND")
        .write_stdin("FOO=bar\n")
        .assert()
        .failure()
        .code(1)
        .stdout("");
}

#[test]
fn test_get_comment_with_piped_input_key_exists_with_comment() {
    envq_cmd()
        .arg("get")
        .arg("comment")
        .arg("FOO")
        .write_stdin("FOO=bar # this is a comment\n")
        .assert()
        .success()
        .code(0)
        .stdout("this is a comment\n");
}

#[test]
fn test_get_comment_with_piped_input_key_exists_no_comment() {
    envq_cmd()
        .arg("get")
        .arg("comment")
        .arg("FOO")
        .write_stdin("FOO=bar\n")
        .assert()
        .success()
        .code(0)
        .stdout("");
}

#[test]
fn test_get_comment_with_piped_input_key_not_found() {
    envq_cmd()
        .arg("get")
        .arg("comment")
        .arg("NOTFOUND")
        .write_stdin("FOO=bar\n")
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_list_default_shows_values() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("list")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("FOO=bar"))
        .stdout(predicate::str::contains("BAR=baz"))
        .stdout(predicate::str::contains("QUX=value"));
}

#[test]
fn test_list_keys_shows_only_keys() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("list")
        .arg("keys")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("FOO\nBAR\nQUX\n");
}

#[test]
fn test_list_values_shows_key_value_pairs() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("list")
        .arg("values")
        .arg(&file_path)
        .assert()
        .success()
        .stdout(predicate::str::contains("FOO=bar"))
        .stdout(predicate::str::contains("BAR=baz"))
        .stdout(predicate::str::contains("QUX=value"));
}

#[test]
fn test_get_error_message_missing_arguments() {
    envq_cmd()
        .arg("get")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "You need to provide what to get [key|comment|header]",
        ))
        .stderr(predicate::str::contains("Example: envq get key FOO"));
}

#[test]
fn test_get_comment_error_message_missing_key() {
    envq_cmd()
        .arg("get")
        .arg("comment")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "You need to provide the name of key",
        ))
        .stderr(predicate::str::contains("Example: envq get comment FOO"));
}

#[test]
fn test_set_error_message_missing_arguments() {
    envq_cmd()
        .arg("set")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "You need to provide what to set [key|comment|header]",
        ))
        .stderr(predicate::str::contains("Example: envq set key FOO VALUE"));
}

#[test]
fn test_get_key_explicit_with_file() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("get")
        .arg("key")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .success()
        .code(0)
        .stdout("bar\n");
}

#[test]
fn test_get_implicit_key_with_file() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("get")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .success()
        .code(0)
        .stdout("bar\n");
}

#[test]
fn test_get_header_with_file() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("get")
        .arg("header")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("Test header\n");
}

#[test]
fn test_set_key_updates_value() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("set")
        .arg("FOO")
        .arg("newvalue")
        .arg(&file_path)
        .assert()
        .success();

    // verify the value was updated
    envq_cmd()
        .arg("get")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("newvalue\n");
}

#[test]
fn test_set_comment_adds_comment() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("set")
        .arg("comment")
        .arg("FOO")
        .arg("new comment")
        .arg(&file_path)
        .assert()
        .success();

    // verify the comment was added
    envq_cmd()
        .arg("get")
        .arg("comment")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("new comment\n");
}

#[test]
fn test_set_header_updates_header() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("set")
        .arg("header")
        .arg("New header\nLine 2")
        .arg(&file_path)
        .assert()
        .success();

    // Verify the header was updated
    envq_cmd()
        .arg("get")
        .arg("header")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("New header\nLine 2\n");
}

#[test]
fn test_del_key_removes_key() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("del")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .success();

    // Verify the key was removed
    envq_cmd()
        .arg("get")
        .arg("FOO")
        .arg(&file_path)
        .assert()
        .failure()
        .code(1);
}

#[test]
fn test_del_comment_removes_comment() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("del")
        .arg("comment")
        .arg("BAR")
        .arg(&file_path)
        .assert()
        .success();

    // verify the comment was removed but key still exists
    envq_cmd()
        .arg("get")
        .arg("BAR")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("baz\n");

    envq_cmd()
        .arg("get")
        .arg("comment")
        .arg("BAR")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_del_header_removes_header() {
    let dir = TempDir::new().unwrap();
    let file_path = create_test_env(&dir);

    envq_cmd()
        .arg("del")
        .arg("header")
        .arg(&file_path)
        .assert()
        .success();

    // verify the header was removed
    envq_cmd()
        .arg("get")
        .arg("header")
        .arg(&file_path)
        .assert()
        .success()
        .stdout("");
}

#[test]
fn test_set_with_stdin_output() {
    envq_cmd()
        .arg("set")
        .arg("FOO")
        .arg("newvalue")
        .write_stdin("FOO=bar\nBAR=baz\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("FOO=newvalue"));
}

#[test]
fn test_del_with_stdin_output() {
    envq_cmd()
        .arg("del")
        .arg("FOO")
        .write_stdin("FOO=bar\nBAR=baz\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("BAR=baz"))
        .stdout(predicate::str::contains("FOO=").not());
}
