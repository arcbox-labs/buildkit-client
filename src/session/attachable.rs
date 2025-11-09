//! Session attachable implementation for BuildKit

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

use super::{FileSyncServer, AuthServer};
use crate::proto::moby::filesync::v1::{
    file_sync_server::FileSyncServer as FileSyncService,
    auth_server::AuthServer as AuthService,
};

/// Session attachable that runs services BuildKit can connect to
pub struct SessionAttachable {
    file_sync: Option<FileSyncServer>,
    auth: Option<AuthServer>,
    port: u16,
    server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<Result<()>>>>>,
}

impl SessionAttachable {
    /// Create a new session attachable
    pub fn new(port: u16) -> Self {
        Self {
            file_sync: None,
            auth: None,
            port,
            server_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Add file sync for a directory
    pub fn with_file_sync(mut self, root_path: PathBuf) -> Self {
        self.file_sync = Some(FileSyncServer::new(root_path));
        self
    }

    /// Add auth server
    pub fn with_auth(mut self, auth: AuthServer) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Start the session services in background
    pub async fn start(&self) -> Result<String> {
        let addr = format!("0.0.0.0:{}", self.port).parse()
            .context("Failed to parse address")?;

        tracing::info!("Starting session attachable on 0.0.0.0:{}", self.port);

        let file_sync = self.file_sync.clone();
        let auth = self.auth.clone();

        // Start gRPC server in background
        let handle = tokio::spawn(async move {
            let mut builder = Server::builder();

            let router = if let Some(fs) = file_sync {
                tracing::debug!("Registered FileSync service");
                let r = builder.add_service(FileSyncService::new(fs));
                if let Some(a) = auth {
                    tracing::debug!("Registered Auth service");
                    r.add_service(AuthService::new(a))
                } else {
                    r
                }
            } else if let Some(a) = auth {
                tracing::debug!("Registered Auth service");
                builder.add_service(AuthService::new(a))
            } else {
                anyhow::bail!("No services registered");
            };

            router
                .serve(addr)
                .await
                .context("Failed to serve session attachable")?;

            Ok(())
        });

        // Store the handle
        let mut server_handle = self.server_handle.lock().await;
        *server_handle = Some(handle);

        // Wait a moment for server to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Return the address that BuildKit can connect to (from inside Docker)
        // Use host.docker.internal which Docker provides to access host from container
        Ok(format!("host.docker.internal:{}", self.port))
    }

    /// Stop the session services
    pub async fn stop(&self) {
        let mut handle = self.server_handle.lock().await;
        if let Some(h) = handle.take() {
            h.abort();
            tracing::info!("Session attachable stopped");
        }
    }
}

impl Drop for SessionAttachable {
    fn drop(&mut self) {
        // Best effort cleanup
        if let Ok(mut handle) = self.server_handle.try_lock() {
            if let Some(h) = handle.take() {
                h.abort();
            }
        }
    }
}
