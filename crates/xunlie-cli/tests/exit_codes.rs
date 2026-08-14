#![forbid(unsafe_code)]

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn missing_input_is_exit_10() {
    Command::cargo_bin("xunlie")
        .unwrap()
        .args(["validate", "does-not-exist.contract.json"])
        .assert()
        .code(10)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("XUNLIE-E010"));
}

#[test]
fn malformed_contract_is_exit_12_and_json_stays_on_stderr() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let path = fixture.write("invalid.json", b"not json").unwrap();

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("validate")
        .arg(path)
        .args(["--format", "json"])
        .assert()
        .code(12)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("\"code\":\"XUNLIE-E012\""));
}

#[test]
fn unknown_command_is_usage_exit_2() {
    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("unknown")
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn compile_then_validate_is_an_end_to_end_success() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture.write_minimal_source().unwrap();
    let contract = fixture.path().join("contract.json");

    let compiled = Command::cargo_bin("xunlie")
        .unwrap()
        .arg("compile")
        .arg(source)
        .arg("--out")
        .arg(&contract)
        .args(["--format", "json"])
        .output()
        .unwrap();

    assert_eq!(compiled.status.code(), Some(0));
    assert!(compiled.stderr.is_empty());
    let result: serde_json::Value = serde_json::from_slice(&compiled.stdout).unwrap();
    assert_eq!(result["status"], "ok");
    assert!(
        result["contentDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert!(
        result["artifactDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    let contract_bytes = std::fs::read(&contract).unwrap();
    xunlie_testkit::assert_schema_version(&contract_bytes, "xunlie.contract/v1");

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("validate")
        .arg(contract)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("valid "))
        .stderr(predicate::str::is_empty());
}

#[test]
fn tampered_provenance_is_exit_12() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture.write_minimal_source().unwrap();
    let contract = fixture.path().join("contract.json");

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("compile")
        .arg(source)
        .arg("--out")
        .arg(&contract)
        .assert()
        .success();

    let mut document: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&contract).unwrap()).unwrap();
    document["sources"][0]["identity"] = serde_json::Value::String("tampered://source".into());
    std::fs::write(&contract, serde_json::to_vec(&document).unwrap()).unwrap();

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("validate")
        .arg(contract)
        .args(["--format", "json"])
        .assert()
        .code(12)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("\"code\":\"XUNLIE-E012\""));
}

#[test]
fn rejected_source_is_exit_11_with_compiler_diagnostics() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture
        .write("invalid-source.json", xunlie_testkit::INVALID_SOURCE_JSON)
        .unwrap();
    let contract = fixture.path().join("contract.json");

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("compile")
        .arg(source)
        .arg("--out")
        .arg(&contract)
        .args(["--format", "json"])
        .assert()
        .code(11)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("\"code\":\"XUNLIE-E011\"")
                .and(predicate::str::contains("\"diagnostics\"")),
        );

    assert!(
        !contract.exists(),
        "a failed compile must not leave partial IR"
    );
}

#[test]
fn unwritable_output_is_exit_13() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture.write_minimal_source().unwrap();

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("compile")
        .arg(source)
        .arg("--out")
        .arg(fixture.path())
        .assert()
        .code(13)
        .stdout(predicate::str::is_empty())
        .stderr(predicate::str::contains("XUNLIE-E013"));
}
