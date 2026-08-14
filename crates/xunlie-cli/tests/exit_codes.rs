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

#[test]
fn variant_then_verify_is_an_end_to_end_success() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture
        .write("history.json", xunlie_testkit::INDEPENDENT_ADDS_SOURCE_JSON)
        .unwrap();
    let variant = fixture.path().join("variant.json");

    let generated = Command::cargo_bin("xunlie")
        .unwrap()
        .arg("variant")
        .arg(&source)
        .args(["--operator", "reverse-independent-adds", "--out"])
        .arg(&variant)
        .args(["--format", "json"])
        .output()
        .unwrap();

    assert_eq!(generated.status.code(), Some(0));
    assert!(generated.stderr.is_empty());
    let result: serde_json::Value = serde_json::from_slice(&generated.stdout).unwrap();
    assert_eq!(result["command"], "variant");
    assert_eq!(result["operator"], "history.independent-adds.reverse");
    assert_eq!(result["contentDigest"].as_str().unwrap().len(), 71);
    assert_eq!(result["certificateDigest"].as_str().unwrap().len(), 71);

    let artifact: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&variant).unwrap()).unwrap();
    assert_eq!(artifact["schemaVersion"], "xunlie.certified-variant/v1");
    assert!(
        artifact["contentDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(artifact["producer"]["name"], "xunlie-engine");
    assert_eq!(
        artifact["certificate"]["schemaVersion"],
        "xunlie.equivalence-certificate/v1"
    );
    assert!(
        artifact["certificate"]["contentDigest"]
            .as_str()
            .unwrap()
            .starts_with("sha256:")
    );
    assert_eq!(
        artifact["certificate"]["before"]["contentDigest"],
        artifact["certificate"]["after"]["contentDigest"]
    );

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("verify-variant")
        .arg(source)
        .arg(variant)
        .args(["--format", "json"])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("\"command\":\"verify-variant\""))
        .stderr(predicate::str::is_empty());
}

#[test]
fn failed_precondition_is_exit_14_and_writes_no_artifact() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let value: serde_json::Value =
        serde_json::from_str(xunlie_testkit::MINIMAL_SOURCE_JSON).unwrap();
    let canonical = serde_json::to_string(&value).unwrap();
    let source = fixture.write("canonical.json", canonical).unwrap();
    let variant = fixture.path().join("variant.json");

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("variant")
        .arg(source)
        .args(["--operator", "normalize-json", "--out"])
        .arg(&variant)
        .args(["--format", "json"])
        .assert()
        .code(14)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("\"code\":\"XUNLIE-E014\"")
                .and(predicate::str::contains("output-differs")),
        );

    assert!(
        !variant.exists(),
        "an excluded variant must not leave a partial artifact"
    );
}

#[test]
fn tampered_certified_variant_is_exit_15() {
    let fixture = xunlie_testkit::FixtureWorkspace::new().unwrap();
    let source = fixture.write_minimal_source().unwrap();
    let variant = fixture.path().join("variant.json");

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("variant")
        .arg(&source)
        .args(["--operator", "normalize-json", "--out"])
        .arg(&variant)
        .assert()
        .success();

    let mut artifact: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&variant).unwrap()).unwrap();
    artifact["sources"][0]["source"] =
        serde_json::Value::String(xunlie_testkit::MINIMAL_SOURCE_JSON.to_owned());
    std::fs::write(&variant, serde_json::to_vec(&artifact).unwrap()).unwrap();

    Command::cargo_bin("xunlie")
        .unwrap()
        .arg("verify-variant")
        .arg(source)
        .arg(variant)
        .args(["--format", "json"])
        .assert()
        .code(15)
        .stdout(predicate::str::is_empty())
        .stderr(
            predicate::str::contains("\"code\":\"XUNLIE-E015\"")
                .and(predicate::str::contains("VARIANT-DIGEST-MISMATCH")),
        );
}
