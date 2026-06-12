use std::process::Command;
use assert_cmd::prelude::*;

#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--help");
    cmd.assert().success().stdout(predicates::str::contains("Usage: miner-cli"));
}

#[test]
fn test_cli_dev_fee_info() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--dev-fee-info");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Pearl Miner Developer Fee Information"))
        .stdout(predicates::str::contains("Default fee: 1%"));
}

#[test]
fn test_cli_missing_args() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode").arg("native-cpu");
    // Missing wallet, worker, pool should fail validation.
    // Tracing logs go to stdout by default in our setup.
    cmd.assert().failure().stdout(predicates::str::contains("Configuration validation failed"));
}

#[test]
fn test_cli_native_cpu_dry_run() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode").arg("native-cpu")
       .arg("--wallet").arg("test_wallet")
       .arg("--worker").arg("test_worker")
       .arg("--pool").arg("test_pool")
       .arg("--dry-run");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Running in native-cpu mode"))
        .stdout(predicates::str::contains("Dry run enabled, exiting"));
}
