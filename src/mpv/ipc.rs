use anyhow::{Context, Result};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::{mpsc, oneshot};

#[cfg(unix)]
use tokio::net::UnixStream;

struct IpcInner {
    cmd_tx: mpsc::Sender<IpcCommand>,
}

struct IpcCommand {
    payload: String,
    request_id: u64,
    reply: oneshot::Sender<Result<Value>>,
}

pub struct MpvIpc {
    inner: IpcInner,
    event_rx: mpsc::Receiver<Value>,
    next_id: u64,
}

impl MpvIpc {
    #[cfg(unix)]
    pub async fn connect(path: &str) -> Result<Self> {
        let stream = UnixStream::connect(path)
            .await
            .with_context(|| format!("Failed to connect to mpv socket at {path}"))?;
        let (reader, writer) = stream.into_split();
        Self::from_parts(BufReader::new(reader), writer)
    }

    #[cfg(windows)]
    pub async fn connect(path: &str) -> Result<Self> {
        let pipe_path = if path.starts_with(r"\\.\pipe\") {
            path.to_string()
        } else {
            format!(r"\\.\pipe\{path}")
        };
        let client = tokio::net::windows::named_pipe::ClientOptions::new()
            .open(&pipe_path)
            .with_context(|| format!("Failed to connect to mpv named pipe at {pipe_path}"))?;
        let (reader, writer) = tokio::io::split(client);
        Self::from_parts(BufReader::new(reader), writer)
    }

    fn from_parts<R, W>(reader: BufReader<R>, mut writer: W) -> Result<Self>
    where
        R: tokio::io::AsyncRead + Unpin + Send + 'static,
        W: tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<IpcCommand>(32);
        let (event_tx, event_rx) = mpsc::channel::<Value>(64);
        let (response_tx, mut response_rx) =
            mpsc::channel::<(u64, oneshot::Sender<Result<Value>>)>(32);

        // Writer task: serialize commands to the socket
        tokio::spawn(async move {
            while let Some(cmd) = cmd_rx.recv().await {
                let _ = response_tx.send((cmd.request_id, cmd.reply)).await;
                let line = format!("{}\n", cmd.payload);
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
        });

        // Reader task: read lines, dispatch responses vs events
        tokio::spawn(async move {
            let mut reader = reader;
            let mut line = String::new();
            let mut pending: Vec<(u64, oneshot::Sender<Result<Value>>)> = Vec::new();

            loop {
                // Drain any newly registered pending replies
                while let Ok(entry) = response_rx.try_recv() {
                    pending.push(entry);
                }

                line.clear();
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {}
                        }
                    }
                    entry = response_rx.recv() => {
                        if let Some(entry) = entry {
                            pending.push(entry);
                        }
                        continue;
                    }
                }

                let parsed: Value = match serde_json::from_str(line.trim()) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if let Some(rid) = parsed.get("request_id").and_then(|v| v.as_u64()) {
                    if let Some(idx) = pending.iter().position(|(id, _)| *id == rid) {
                        let (_, reply) = pending.swap_remove(idx);
                        if let Some(err) = parsed.get("error").and_then(|e| e.as_str()) {
                            if err != "success" {
                                let _ = reply.send(Err(anyhow::anyhow!("mpv error: {err}")));
                                continue;
                            }
                        }
                        let data = parsed.get("data").cloned().unwrap_or(Value::Null);
                        let _ = reply.send(Ok(data));
                    }
                } else if parsed.get("event").is_some() {
                    let _ = event_tx.try_send(parsed);
                }
            }
        });

        Ok(Self {
            inner: IpcInner { cmd_tx },
            event_rx,
            next_id: 1,
        })
    }

    pub async fn command(&mut self, args: &[Value]) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;

        let payload = serde_json::json!({
            "command": args,
            "request_id": id,
        });

        let (tx, rx) = oneshot::channel();
        self.inner
            .cmd_tx
            .send(IpcCommand {
                payload: payload.to_string(),
                request_id: id,
                reply: tx,
            })
            .await
            .context("mpv IPC channel closed")?;

        rx.await.context("mpv response channel dropped")?
    }

    pub async fn get_property(&mut self, name: &str) -> Result<Value> {
        self.command(&[
            Value::String("get_property".into()),
            Value::String(name.into()),
        ])
        .await
    }

    pub async fn set_property(&mut self, name: &str, value: Value) -> Result<()> {
        self.command(&[
            Value::String("set_property".into()),
            Value::String(name.into()),
            value,
        ])
        .await?;
        Ok(())
    }

    pub async fn observe_property(&mut self, id: u64, name: &str) -> Result<()> {
        self.command(&[
            Value::String("observe_property".into()),
            Value::Number(id.into()),
            Value::String(name.into()),
        ])
        .await?;
        Ok(())
    }

    pub async fn recv_event(&mut self) -> Option<Value> {
        self.event_rx.recv().await
    }
}
