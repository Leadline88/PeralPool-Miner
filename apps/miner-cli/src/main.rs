mod args;
mod benchmark;
mod commands;
mod config_loader;
mod dev_fee_info;
mod modes;
mod output;

use clap::Parser;

#[tokio::main]
async fn main() {
    logging::init();
    let args = args::Args::parse();
    let config = config_loader::load_config(&args);

    commands::dispatch(config, args).await;
}
