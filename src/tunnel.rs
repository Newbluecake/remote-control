use anyhow::{bail, Context, Result};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tracing::{error, info};

pub struct CloudflareTunnel {
    child: Child,
}

impl CloudflareTunnel {
    pub async fn start(local_port: u16) -> Result<Self> {
        let url = format!("http://localhost:{local_port}");

        let mut child = Command::new("cloudflared")
            .args(["tunnel", "--url", &url])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .context("Failed to start cloudflared. Is it installed?")?;

        let stderr = child.stderr.take().unwrap();
        let mut reader = BufReader::new(stderr).lines();

        // cloudflared prints the tunnel URL to stderr
        let tunnel_url = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            async {
                while let Some(line) = reader.next_line().await? {
                    if let Some(url) = extract_tunnel_url(&line) {
                        return Ok::<String, anyhow::Error>(url);
                    }
                }
                bail!("cloudflared exited without providing a tunnel URL")
            },
        )
        .await
        .context("Timed out waiting for cloudflared tunnel URL")??;

        info!("Cloudflare Tunnel: {tunnel_url}");

        // Drain remaining stderr in background to avoid blocking cloudflared
        tokio::spawn(async move {
            while let Ok(Some(line)) = reader.next_line().await {
                if line.contains("ERR") {
                    error!("cloudflared: {line}");
                }
            }
        });

        Ok(Self { child })
    }

    pub async fn shutdown(mut self) {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        info!("Cloudflare Tunnel closed");
    }
}

fn extract_tunnel_url(line: &str) -> Option<String> {
    // cloudflared logs: "... https://xxx-xxx-xxx.trycloudflare.com ..."
    line.split_whitespace()
        .find(|word| word.starts_with("https://") && word.contains(".trycloudflare.com"))
        .map(|s| s.to_string())
}
