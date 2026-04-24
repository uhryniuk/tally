use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn tally(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("tally").unwrap();
    cmd.env("HOME", home.path());
    cmd
}

#[test]
fn version_flag_prints_package_version() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn add_persists_and_reads_back() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["foo", "add", "5"]).assert().success();
    tally(&home)
        .args(["foo"])
        .assert()
        .success()
        .stdout("5\n");
}

#[test]
fn sub_decrements_counter() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["foo", "add", "10"]).assert().success();
    tally(&home).args(["foo", "sub", "3"]).assert().success();
    tally(&home)
        .args(["foo"])
        .assert()
        .success()
        .stdout("7\n");
}

#[test]
fn add_without_amount_uses_step() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["foo", "set", "--step", "4"])
        .assert()
        .success();
    tally(&home).args(["foo", "add"]).assert().success();
    tally(&home).args(["foo", "add"]).assert().success();
    tally(&home)
        .args(["foo"])
        .assert()
        .success()
        .stdout("8\n");
}

#[test]
fn set_without_default_flag_preserves_default() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["foo", "add", "1"]).assert().success();
    tally(&home)
        .args(["foo", "set", "--step", "3"])
        .assert()
        .success();

    let output = tally(&home)
        .args(["list", "--no-headers"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();

    // The built-in "tally" counter created at init should still be the default.
    let tally_line = text
        .lines()
        .find(|l| l.starts_with("tally"))
        .expect("tally row present");
    let foo_line = text
        .lines()
        .find(|l| l.starts_with("foo"))
        .expect("foo row present");
    assert!(
        tally_line.trim_end().ends_with('*'),
        "tally should still be default: {tally_line:?}"
    );
    assert!(
        !foo_line.trim_end().ends_with('*'),
        "foo should not have been promoted: {foo_line:?}"
    );
}

#[test]
fn set_with_default_promotes_counter() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["foo", "set", "--default"])
        .assert()
        .success();

    let output = tally(&home)
        .args(["list", "--no-headers"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    let foo_line = text
        .lines()
        .find(|l| l.starts_with("foo"))
        .expect("foo row present");
    assert!(foo_line.trim_end().ends_with('*'));
}

#[test]
fn list_no_headers_skips_header_row() {
    let home = TempDir::new().unwrap();
    let output = tally(&home)
        .args(["list", "--no-headers"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    assert!(
        !text.contains("Name"),
        "--no-headers should hide the header row: {text:?}"
    );
}

#[test]
fn list_includes_headers_by_default() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Name"))
        .stdout(predicate::str::contains("Count"));
}

#[test]
fn delete_removes_counter() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["foo", "add", "1"]).assert().success();
    tally(&home).args(["foo", "delete"]).assert().success();

    let output = tally(&home)
        .args(["list", "--no-headers"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).unwrap();
    assert!(!text.lines().any(|l| l.starts_with("foo")), "{text:?}");
}

#[test]
fn nuke_with_yes_removes_database() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["foo", "add", "1"]).assert().success();
    let db = home.path().join(".tally").join("tally.db");
    assert!(db.exists());
    tally(&home).args(["nuke", "--yes"]).assert().success();
    assert!(!db.exists());
}

#[test]
fn quiet_suppresses_stdout() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["--quiet", "foo", "add", "1"])
        .assert()
        .success()
        .stdout("");
}

#[test]
fn template_renders_nested_reference() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["inner", "set", "5"])
        .assert()
        .success();
    tally(&home)
        .args(["outer", "set", "--template", "[{inner}]"])
        .assert()
        .success();
    tally(&home)
        .args(["outer"])
        .assert()
        .success()
        .stdout("[5]\n");
}

#[test]
fn raw_skips_template() {
    let home = TempDir::new().unwrap();
    tally(&home)
        .args(["foo", "set", "5", "--template", "count={}"])
        .assert()
        .success();
    tally(&home)
        .args(["--raw", "foo"])
        .assert()
        .success()
        .stdout("5\n");
}

#[test]
fn bare_invocation_after_nuke_still_works() {
    let home = TempDir::new().unwrap();
    tally(&home).args(["nuke", "--yes"]).assert().success();
    tally(&home).args(["foo", "add", "1"]).assert().success();
}
