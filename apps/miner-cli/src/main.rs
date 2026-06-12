use config::Config;
use std::process::exit;
use tracing::{error, info};
use watchdog::Watchdog;

#[tokio::main]
async fn main() {
    logging::init();
    info!("Starting Pearl Miner CLI manager...");

    let config_path = "config.toml";
    let config = match Config::load_from_file(config_path) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to load configuration from {}: {}", config_path, e);
            exit(1);
        }
    };

    info!("Loaded configuration for worker: {}", config.worker_name);

    Watchdog::run(config.miner_binary_path, config.args).await;
}
