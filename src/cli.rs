use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "remote-control",
    version,
    about = "Synchronized keyboard control over the network"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the WebSocket relay server
    Serve(ServeArgs),
    /// Join a room and sync keyboard events with peers
    Join(JoinArgs),
}

#[derive(clap::Args)]
pub struct ServeArgs {
    /// Address to bind the server to
    #[arg(short, long, default_value = "0.0.0.0:9090")]
    pub bind: String,
}

#[derive(clap::Args)]
pub struct JoinArgs {
    /// WebSocket server URL
    #[arg(short = 'S', long, default_value = "ws://localhost:9090")]
    pub server: String,

    /// Room code to join (auto-generated if not provided)
    #[arg(short, long)]
    pub room: Option<String>,

    /// Your nickname
    #[arg(short, long, default_value = "anon")]
    pub nickname: String,

    /// Start with keyboard sync disabled (toggle with Ctrl+Shift+F12)
    #[arg(long)]
    pub no_sync: bool,
}
