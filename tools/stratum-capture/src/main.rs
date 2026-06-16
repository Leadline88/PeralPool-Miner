mod args;
mod error;
mod fixture;
mod jsonrpc;
mod proxy;
mod redaction;

use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = args::Args::parse();

    if let Err(e) = args.validate() {
        eprintln!("Configuration error: {}", e);
        std::process::exit(1);
    }

    if args.dry_run {
        println!("Dry run complete. Arguments are valid.");
        return Ok(());
    }

    if let Err(e) = proxy::run(args).await {
        eprintln!("Runtime error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}
