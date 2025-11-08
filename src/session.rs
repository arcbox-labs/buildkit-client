//! BuildKit session implementation for file access and streaming

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tonic::transport::Channel;
use uuid::Uuid;

use crate::proto::moby::buildkit::v1::{BytesMessage, control_client::ControlClient};

/// Session manager for BuildKit
pub struct Session {
    pub id: String,
    pub shared_key: String,
    tx: Option<mpsc::Sender<BytesMessage>>,
}

impl Session {
    /// Create a new session
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        let shared_key = format!("session-{}", Uuid::new_v4());

        Self {
            id,
            shared_key,
            tx: None,
        }
    }

    /// Start a session with BuildKit
    pub async fn start(&mut self, control: ControlClient<Channel>) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<BytesMessage>(32);

        // Clone the control client for the session
        let mut session_control = control.clone();

        // Create the bidirectional stream
        let outbound = async_stream::stream! {
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        };

        // Start the session
        let response = session_control
            .session(outbound)
            .await
            .context("Failed to start session")?;

        let mut inbound = response.into_inner();

        // Spawn a task to handle inbound messages
        tokio::spawn(async move {
            while let Ok(Some(msg)) = inbound.message().await {
                // Handle inbound session messages
                tracing::debug!("Received session message: {} bytes", msg.data.len());
                // In a full implementation, we would:
                // 1. Parse the message to determine the method being called
                // 2. Handle file sync requests (ReadDir, StatFile, etc.)
                // 3. Send responses back through the stream
            }
        });

        self.tx = Some(tx);
        Ok(())
    }

    /// Get session metadata to attach to solve request
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("X-Docker-Expose-Session-Uuid".to_string(), self.id.clone());
        meta.insert("X-Docker-Expose-Session-Name".to_string(), self.shared_key.clone());
        meta.insert("X-Docker-Expose-Session-Sharedkey".to_string(), self.shared_key.clone());
        meta
    }

    /// Send a message to the session stream
    pub async fn send(&self, msg: BytesMessage) -> Result<()> {
        if let Some(ref tx) = self.tx {
            tx.send(msg)
                .await
                .context("Failed to send message to session")?;
            Ok(())
        } else {
            anyhow::bail!("Session not started");
        }
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// File sync helper for sending local files to BuildKit
pub struct FileSync {
    context_path: PathBuf,
}

impl FileSync {
    pub fn new(context_path: impl Into<PathBuf>) -> Self {
        Self {
            context_path: context_path.into(),
        }
    }

    /// Check if path exists and is accessible
    pub fn validate(&self) -> Result<()> {
        if !self.context_path.exists() {
            anyhow::bail!("Context path does not exist: {}", self.context_path.display());
        }
        if !self.context_path.is_dir() {
            anyhow::bail!("Context path is not a directory: {}", self.context_path.display());
        }
        Ok(())
    }

    /// Get absolute path
    pub fn absolute_path(&self) -> Result<PathBuf> {
        std::fs::canonicalize(&self.context_path)
            .context("Failed to resolve absolute path")
    }
}
