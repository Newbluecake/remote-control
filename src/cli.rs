use clap::{Parser, Subcommand};

const fn default_mpv_socket() -> &'static str {
    if cfg!(windows) {
        r"\\.\pipe\mpvsocket"
    } else {
        "/tmp/mpvsocket"
    }
}

#[derive(Parser)]
#[command(
    name = "remote-control",
    version,
    about = "Synchronized movie watching"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the WebSocket relay server
    Serve(ServeArgs),
    /// Join a room and sync with a partner
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

    /// Path to mpv IPC socket (Unix) or named pipe name (Windows)
    #[arg(short = 's', long, default_value = default_mpv_socket())]
    pub mpv_socket: String,

    /// Drift correction threshold in seconds
    #[arg(long, default_value = "0.5")]
    pub drift_threshold: f64,
}
