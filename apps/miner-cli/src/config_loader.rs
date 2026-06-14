use crate::args::Args;
use config::Config;
use std::process::exit;
use tracing::error;

pub fn load_config(args: &Args) -> Config {
    let path = &args.config;
    let mut config = if path.exists() {
        match Config::load_from_file(path) {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to load configuration from {:?}: {}", path, e);
                exit(1);
            }
        }
    } else {
        Config::default()
    };

    apply_cli_overrides(&mut config, args);
    config
}

fn apply_cli_overrides(config: &mut Config, args: &Args) {
    // Process profiles if specified
    if let Some(profile_name) = &args.profile {
        if let Some(profiles) = &config.profiles {
            if let Some(profile) = profiles.get(profile_name) {
                config.pool_url = profile.pool_url.clone();
                config.algo = profile.algo.clone();

                if let Some(miners) = &config.miners {
                    if let Some(miner_config) = miners.get(&profile.miner) {
                        config.miner_binary_path = miner_config.binary_path.clone();
                        config.args = miner_config.expand_args(
                            &config.wallet,
                            &config.worker_name,
                            &config.pool_url,
                            &config.algo,
                        );
                    }
                }
            } else {
                error!("Profile '{}' not found in configuration.", profile_name);
                exit(1);
            }
        } else {
            error!("No profiles found in configuration.");
            exit(1);
        }
    }

    // Override config with CLI arguments if provided
    if let Some(wallet) = args.wallet.as_ref() {
        config.wallet = wallet.clone();
    }
    if let Some(worker) = args.worker.as_ref() {
        config.worker_name = worker.clone();
    }
    if let Some(pool) = args.pool.as_ref() {
        config.pool_url = pool.clone();
    }
    if let Some(miner_binary) = args.miner_binary.as_ref() {
        config.miner_binary_path = miner_binary.clone();
    }
    if let Some(mode) = args.mode {
        config.mode = mode.into();
    }
    if let Some(backend) = args.backend {
        config.backend = backend.into();
    }
    if let Some(algo) = args.algo.as_ref() {
        config.algo = algo.clone();
    }
    if let Some(threads) = args.threads {
        config.threads = threads;
    }
    if args.deterministic {
        config.deterministic = true;
    }
    if args.dry_run {
        config.dry_run = true;
    }
    if args.benchmark_native_cpu {
        config.benchmark = true;
    }
}
