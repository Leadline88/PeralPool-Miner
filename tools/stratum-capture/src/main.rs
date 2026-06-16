mod args;
mod fixture;
mod jsonrpc;
mod proxy;
mod redaction;

use clap::Parser;

#[tokio::main]
async fn main() {
    let args = args::Args::parse();
    args.validate();

    if args.dry_run {
        println!("Dry run complete. Arguments are valid.");
        return;
    }

    if let Err(e) = proxy::run(args).await {
        eprintln!("Error: {}", e);
    }
}
