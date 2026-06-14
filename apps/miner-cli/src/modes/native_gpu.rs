use config::Config;
use std::process::exit;
use tracing::{error, info};

pub async fn run(config: Config) {
    info!("Running in native-gpu mode");
    info!("Backend: {:?}", config.backend);
    error!(
        "Native GPU mining is not yet implemented for {:?} backend.",
        config.backend
    );
    exit(1);
}
