//! gRPC server for session services

use anyhow::{Context, Result};
use std::net::SocketAddr;
use std::path::PathBuf;
use tonic::transport::Server;

use super::{FileSyncServer, AuthServer};
use crate::proto::moby::filesync::v1::{
    file_sync_server::FileSyncServer as FileSyncService,
    auth_server::AuthServer as AuthService,
};

/// Session gRPC server
pub struct SessionServer {
    addr: SocketAddr,
    file_sync: Option<FileSyncServer>,
    auth: Option<AuthServer>,
}

impl SessionServer {
    /// Create a new session server
    pub fn new(port: u16) -> Self {
        Self {
            addr: ([127, 0, 0, 1], port).into(),
            file_sync: None,
            auth: None,
        }
    }

    /// Add file sync service
    pub fn with_file_sync(mut self, root_path: PathBuf) -> Self {
        self.file_sync = Some(FileSyncServer::new(root_path));
        self
    }

    /// Add auth service
    pub fn with_auth(mut self, auth: AuthServer) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Start the gRPC server
    pub async fn serve(self) -> Result<()> {
        tracing::info!("Starting session gRPC server on {}", self.addr);

        let mut builder = Server::builder();

        // Build the router with services
        let router = if let Some(file_sync) = self.file_sync {
            tracing::debug!("Registered FileSync service");
            let r = builder.add_service(FileSyncService::new(file_sync));
            if let Some(auth) = self.auth {
                tracing::debug!("Registered Auth service");
                r.add_service(AuthService::new(auth))
            } else {
                r
            }
        } else if let Some(auth) = self.auth {
            tracing::debug!("Registered Auth service");
            builder.add_service(AuthService::new(auth))
        } else {
            anyhow::bail!("No services registered");
        };

        router
            .serve(self.addr)
            .await
            .context("Failed to start session server")?;

        Ok(())
    }

    /// Get the address the server will listen on
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}
