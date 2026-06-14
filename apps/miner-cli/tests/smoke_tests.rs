use assert_cmd::prelude::*;
use chrono::Utc;
use pearlpool::PearlPoolAdapter;
use shares::ShareCandidate;
use std::process::Command;
use stratum::mock_adapter::MockPoolAdapter;
use stratum::PoolAdapter;

#[test]
fn test_cli_validate_config_compatibility() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--validate-config")
        .arg("--mode")
        .arg("compatibility")
        .arg("--miner-binary")
        .arg("lpminer.exe")
        .arg("--pool")
        .arg("stratum+tcp://pearlpool.cloud:5566")
        .arg("--wallet")
        .arg("1A1z")
        .arg("--worker")
        .arg("worker1");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("Configuration is valid"));
}

#[test]
fn test_cli_validate_config_native_cpu_offline() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--validate-config").arg("--mode").arg("native-cpu");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("Configuration is valid"));
}

#[test]
fn test_cli_validate_config_native_cpu_experimental_gate() {
    // Should fail without credentials
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--validate-config")
        .arg("--mode")
        .arg("native-cpu")
        .arg("--allow-experimental-live-stratum");

    let output = cmd.output().expect("failed to execute process");
    assert!(!output.status.success());
}

#[test]
fn test_cli_print_command_compatibility() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode")
        .arg("compatibility")
        .arg("--miner-binary")
        .arg("lpminer.exe")
        .arg("--pool")
        .arg("stratum+tcp://pearlpool.cloud:5566")
        .arg("--wallet")
        .arg("1A1z")
        .arg("--print-command");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("External miner command:"));
    assert!(stdout.contains("lpminer.exe"));
}

#[test]
fn test_cli_print_command_restricted() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode").arg("native-cpu").arg("--print-command");

    let output = cmd.output().expect("failed to execute process");
    assert!(!output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("--print-command is only available in compatibility mode"));
}

#[test]
fn test_cli_dry_run_compatibility() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode")
        .arg("compatibility")
        .arg("--miner-binary")
        .arg("non_existent_miner")
        .arg("--dry-run");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run enabled. Not starting external miner."));
}

#[test]
fn test_cli_dry_run_native_cpu() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode").arg("native-cpu").arg("--dry-run");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("Dry run enabled, exiting."));
}

#[test]
fn test_cli_dry_run_experimental_stratum() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode")
        .arg("native-cpu")
        .arg("--allow-experimental-live-stratum")
        .arg("--wallet")
        .arg("1A1z")
        .arg("--pool")
        .arg("stratum+tcp://localhost:5566")
        .arg("--dry-run");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("Dry run enabled. Not connecting to pool."));
}

#[test]
fn test_cli_native_gpu_not_implemented() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode").arg("native-gpu");

    let output = cmd.output().expect("failed to execute process");
    assert!(!output.status.success());
    let combined_output = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(combined_output.contains("Native GPU mining is not yet implemented"));
}

#[test]
fn test_cli_dev_fee_info() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--dev-fee-info");

    let output = cmd.output().expect("failed to execute process");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("active collection is not implemented in native mode"));
}

#[test]
fn test_pearlpool_adapter_rejects_live_submit() {
    let adapter = PearlPoolAdapter::new(true);
    let share = ShareCandidate {
        job_id: "test".to_string(),
        nonce: "123".to_string(),
        result: "hash".to_string(),
        worker: "worker".to_string(),
        timestamp: Utc::now(),
    };

    let res = adapter.build_share_submit(&share);
    assert!(res.is_err());
    assert!(matches!(
        res.unwrap_err(),
        stratum::adapter::PoolAdapterError::UnsupportedRealPearlShareSubmitFormat
    ));
}

#[test]
fn test_mock_adapter_supports_fake_submit() {
    let adapter = MockPoolAdapter;
    let share = ShareCandidate {
        job_id: "test".to_string(),
        nonce: "123".to_string(),
        result: "hash".to_string(),
        worker: "worker".to_string(),
        timestamp: Utc::now(),
    };

    let res = adapter.build_share_submit(&share);
    assert!(res.is_ok());
    let req = res.unwrap();
    assert_eq!(req.method, "mining.submit");
}
