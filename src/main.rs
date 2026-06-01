mod cli;
mod client;
mod keyboard;
mod protocol;
mod relay;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .init();

    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Serve(args) => relay::run_server(&args.bind).await,
        Commands::Join(args) => client::run_client(args).await,
    };

    if let Err(e) = result {
        eprintln!("Error: {e:#}");
        std::process::exit(1);
    }
}
