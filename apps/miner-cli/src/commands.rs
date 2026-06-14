use crate::args::Args;
use crate::modes;
use config::{Config, MiningMode};
use std::process::exit;
use tracing::{error, info};

pub async fn dispatch(config: Config, args: Args) {
    if args.dev_fee_info {
        crate::dev_fee_info::print();
        return;
    }

    if let Some(path) = &args.verify_pearl_fixture {
        verify_fixture(path).await;
        return;
    }

    // Validate merged config
    let validation_res = if args.allow_experimental_live_stratum {
        config.validate_live()
    } else {
        config.validate()
    };

    if let Err(e) = validation_res {
        error!("Configuration validation failed: {}", e);
        error!("Please provide missing values via config file or CLI arguments.");
        exit(1);
    }

    if args.validate_config {
        info!("Configuration is valid.");
        return;
    }

    if args.print_command {
        if config.mode == MiningMode::Compatibility {
            println!("External miner command:");
            println!("{} {}", config.miner_binary_path, config.args.join(" "));
            return;
        } else {
            error!("--print-command is only available in compatibility mode.");
            exit(1);
        }
    }

    info!("Starting Pearl Miner...");
    info!("Mode: {:?}", config.mode);
    info!("Backend: {:?}", config.backend);
    info!("Algorithm: {}", config.algo);
    info!("Worker: {}", config.worker_name);
    info!("Developer fee: 1.0%");

    if config.benchmark {
        crate::benchmark::run(&args).await;
        return;
    }

    match config.mode {
        MiningMode::Compatibility => {
            modes::compatibility::run(config).await;
        }
        MiningMode::NativeCpu => {
            if args.allow_experimental_live_stratum {
                modes::experimental_stratum::run(config).await;
            } else {
                modes::native_cpu::run(config).await;
            }
        }
        MiningMode::NativeGpu => {
            modes::native_gpu::run(config).await;
        }
    }
}

pub async fn verify_fixture(path: &str) {
    info!("Verifying fixture: {}", path);
    let content = std::fs::read_to_string(path).expect("Failed to read fixture");
    let fixture: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse fixture");

    let blob = hex::decode(fixture["blob"].as_str().unwrap()).unwrap();
    let target = fixture["target"].as_u64().unwrap();

    if let Some(expected_shares) = fixture["expected_shares"].as_array() {
        for share in expected_shares {
            let nonce = share["nonce"].as_u64().unwrap();
            let is_valid = algo_pearl::PearlVerifier::verify(&blob, nonce, target);
            if is_valid {
                info!("Fixture Share [nonce: {}] - VALID", nonce);
            } else {
                error!("Fixture Share [nonce: {}] - INVALID", nonce);
                std::process::exit(1);
            }
        }
    }
    info!("Fixture verification successful!");
}
