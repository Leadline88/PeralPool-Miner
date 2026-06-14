mod args;
mod benchmark;
mod commands;
mod config_loader;
mod dev_fee_info;
mod modes;
mod output;

use clap::Parser;
use std::process::exit;
use tracing::error;

#[tokio::main]
async fn main() {
    logging::init();
    let args = args::Args::parse();

    if args.dev_fee_info {
        dev_fee_info::print();
        return;
    }

    if let Some(path) = &args.verify_pearl_fixture {
        commands::verify_fixture(path).await;
        return;
    }

    let config = config_loader::load_config(&args);

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

    commands::dispatch(config, args).await;
}
