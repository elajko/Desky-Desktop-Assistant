use crate::config::Settings;
use serde::Serialize;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum LlmStatus {
    Stopped,
    Starting,
    Ready,
    Crashed { message: String },
}

pub struct LlmProcess {
    child: Option<Child>,
    pub port: u16,
    pub status: LlmStatus,
}

impl Default for LlmProcess {
    fn default() -> Self {
        Self {
            child: None,
            port: 0,
            status: LlmStatus::Stopped,
        }
    }
}

impl LlmProcess {
    /// Spawns llama-server if not already running/starting, and waits until it reports healthy.
    /// Returns the port it's actually listening on.
    pub async fn ensure_running(&mut self, settings: &Settings) -> anyhow::Result<u16> {
        if self.status == LlmStatus::Ready {
            if let Some(child) = &mut self.child {
                if child.try_wait()?.is_none() {
                    return Ok(self.port);
                }
            }
        }

        let server_path = settings
            .llama_server_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("llama-server path not configured in Settings"))?;
        let model_path = settings
            .model_path
            .clone()
            .ok_or_else(|| anyhow::anyhow!("model (.gguf) path not configured in Settings"))?;

        self.status = LlmStatus::Starting;
        let port = pick_port(settings.port);
        self.port = port;

        let child = Command::new(&server_path)
            .arg("--model")
            .arg(&model_path)
            .arg("--port")
            .arg(port.to_string())
            .arg("--host")
            .arg("127.0.0.1")
            .arg("-c")
            .arg(settings.context_size.to_string())
            .arg("--jinja")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| anyhow::anyhow!("failed to spawn llama-server at {server_path:?}: {e}"))?;

        self.child = Some(child);

        self.wait_until_healthy(port).await?;
        self.status = LlmStatus::Ready;
        Ok(port)
    }

    async fn wait_until_healthy(&mut self, port: u16) -> anyhow::Result<()> {
        let url = format!("http://127.0.0.1:{port}/health");
        let client = reqwest::Client::new();
        let mut delay = Duration::from_millis(200);
        let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

        loop {
            if let Some(child) = &mut self.child {
                if let Some(exit_status) = child.try_wait()? {
                    let message = format!("llama-server exited during startup ({exit_status})");
                    self.status = LlmStatus::Crashed {
                        message: message.clone(),
                    };
                    anyhow::bail!(message);
                }
            }

            if let Ok(resp) = client.get(&url).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }

            if tokio::time::Instant::now() >= deadline {
                let message = "timed out waiting for llama-server /health".to_string();
                self.status = LlmStatus::Crashed {
                    message: message.clone(),
                };
                anyhow::bail!(message);
            }

            tokio::time::sleep(delay).await;
            delay = (delay * 2).min(Duration::from_secs(2));
        }
    }

    /// True if the child process exited on its own (crash), distinct from us stopping it.
    pub fn poll_crashed(&mut self) -> Option<String> {
        if self.status != LlmStatus::Ready {
            return None;
        }
        if let Some(child) = &mut self.child {
            if let Ok(Some(exit_status)) = child.try_wait() {
                let message = format!("llama-server exited unexpectedly ({exit_status})");
                self.status = LlmStatus::Crashed {
                    message: message.clone(),
                };
                return Some(message);
            }
        }
        None
    }

    pub async fn shutdown(&mut self) {
        let Some(mut child) = self.child.take() else {
            self.status = LlmStatus::Stopped;
            return;
        };

        // Try a graceful SIGTERM first if we can get the pid, but — unlike
        // before — the forceful kill()+wait() fallback below always runs
        // regardless of whether id() gave us a pid. child.id() can return
        // None even for a still-running child; previously that silently
        // skipped killing the process entirely, orphaning llama-server.
        #[cfg(unix)]
        if let Some(pid) = child.id() {
            unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            }
        }

        if tokio::time::timeout(Duration::from_secs(5), child.wait())
            .await
            .is_err()
        {
            let _ = child.kill().await;
            let _ = child.wait().await;
        }

        self.status = LlmStatus::Stopped;
    }
}

fn pick_port(preferred: u16) -> u16 {
    if std::net::TcpListener::bind(("127.0.0.1", preferred)).is_ok() {
        return preferred;
    }
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|l| l.local_addr())
        .map(|addr| addr.port())
        .unwrap_or(preferred)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use std::path::PathBuf;

    fn llama_server_process_alive(port: u16) -> bool {
        let output = std::process::Command::new("pgrep")
            .arg("-f")
            .arg(format!("llama-server.*--port {port}"))
            .output()
            .expect("pgrep should run");
        !output.stdout.is_empty()
    }

    /// Repro + regression test for the orphaned llama-server bug: shutdown()
    /// used to only attempt to kill the child inside `if let Some(pid) =
    /// child.id()`, silently doing nothing if id() returned None for a still
    /// -running child. This drives the real spawn -> shutdown path and
    /// checks the OS process list directly, independent of whatever made
    /// id() return None in the field.
    /// Run with: cargo test --lib -- --ignored --nocapture shutdown_always_kills_llama_server
    #[ignore]
    #[tokio::test]
    async fn shutdown_always_kills_llama_server() {
        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-3b-instruct-q4_k_m.gguf",
            )),
            port: 8097,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        assert!(
            llama_server_process_alive(port),
            "llama-server should be running on port {port} right after ensure_running"
        );

        llm.shutdown().await;

        // Give the OS a moment to actually reap the process after kill.
        tokio::time::sleep(Duration::from_millis(500)).await;

        assert!(
            !llama_server_process_alive(port),
            "llama-server on port {port} should be gone after shutdown(), but it's still running (orphan bug)"
        );

        println!("PASS: llama-server on port {port} was fully terminated by shutdown()");
    }
}
