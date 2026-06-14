use config::Config;
use tracing::info;
use watchdog::Watchdog;

pub async fn run(config: Config) {
    info!("Running in compatibility mode (external miner)");
    if config.dry_run {
        info!("Dry run enabled. Not starting external miner.");
        return;
    }
    Watchdog::run(config.miner_binary_path, config.args).await;
}
