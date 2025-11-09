//! Simplified session implementation using gRPC server approach

use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

use super::{AuthServer, SessionAttachable};

/// Simplified session manager
pub struct SimpleSession {
    pub id: String,
    pub shared_key: String,
    attachable: Option<SessionAttachable>,
}

impl SimpleSession {
    /// Create a new session
    pub fn new() -> Self {
        let id = Uuid::new_v4().to_string();
        let shared_key = format!("session-{}", Uuid::new_v4());

        Self {
            id,
            shared_key,
            attachable: None,
        }
    }

    /// Add file sync service for a directory
    pub fn add_file_sync(&mut self, root_path: PathBuf) {
        if let Some(attachable) = self.attachable.take() {
            self.attachable = Some(attachable.with_file_sync(root_path));
        } else {
            // Use a random port in the ephemeral range
            let port = 50000 + (rand::random::<u16>() % 10000);
            self.attachable = Some(SessionAttachable::new(port).with_file_sync(root_path));
        }
    }

    /// Add auth service
    pub fn add_auth(&mut self, auth: AuthServer) {
        if let Some(attachable) = self.attachable.take() {
            self.attachable = Some(attachable.with_auth(auth));
        } else {
            let port = 50000 + (rand::random::<u16>() % 10000);
            self.attachable = Some(SessionAttachable::new(port).with_auth(auth));
        }
    }

    /// Start the session
    pub async fn start(&mut self) -> Result<()> {
        if let Some(ref attachable) = self.attachable {
            let addr = attachable.start().await?;
            tracing::info!("Session {} started at {}", self.id, addr);
            Ok(())
        } else {
            tracing::warn!("No services attached to session");
            Ok(())
        }
    }

    /// Get session ID
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    /// Get session metadata
    pub fn metadata(&self) -> HashMap<String, String> {
        let mut meta = HashMap::new();
        meta.insert("X-Docker-Expose-Session-Uuid".to_string(), self.id.clone());
        meta.insert("X-Docker-Expose-Session-Name".to_string(), self.shared_key.clone());
        meta.insert("X-Docker-Expose-Session-Sharedkey".to_string(), self.shared_key.clone());
        meta
    }

    /// Stop the session
    pub async fn stop(&self) {
        if let Some(ref attachable) = self.attachable {
            attachable.stop().await;
        }
    }
}

impl Default for SimpleSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SimpleSession {
    fn drop(&mut self) {
        // Best effort cleanup - stop is handled manually by caller
        tracing::debug!("SimpleSession {} dropped", self.id);
    }
}
