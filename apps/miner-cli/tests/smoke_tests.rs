use assert_cmd::prelude::*;
use chrono::Utc;
use pearlpool::PearlPoolAdapter;
use shares::ShareCandidate;
use std::process::Command;
use stratum::mock_adapter::MockPoolAdapter;
use stratum::PoolAdapter;

#[test]
fn test_cli_validate_config() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--validate-config")
        .arg("--wallet")
        .arg("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa")
        .arg("--pool")
        .arg("stratum+tcp://pearlpool.cloud:5566")
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
fn test_cli_native_gpu_not_implemented() {
    let mut cmd = Command::cargo_bin("miner-cli").unwrap();
    cmd.arg("--mode")
        .arg("native-gpu")
        .arg("--wallet")
        .arg("1A1z")
        .arg("--pool")
        .arg("stratum+tcp://pearlpool.cloud:5566");

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
