//! Authentication protocol implementation for BuildKit sessions

use tonic::{Request, Response, Status};
use crate::proto::moby::filesync::v1::{
    auth_server::Auth,
    CredentialsRequest, CredentialsResponse,
    FetchTokenRequest, FetchTokenResponse,
    GetTokenAuthorityRequest, GetTokenAuthorityResponse,
    VerifyTokenAuthorityRequest, VerifyTokenAuthorityResponse,
};

/// Registry authentication configuration
#[derive(Debug, Clone)]
pub struct RegistryAuthConfig {
    pub host: String,
    pub username: String,
    pub password: String,
}

/// Auth server implementation
#[derive(Debug, Clone, Default)]
pub struct AuthServer {
    registries: Vec<RegistryAuthConfig>,
}

impl AuthServer {
    pub fn new() -> Self {
        Self {
            registries: Vec::new(),
        }
    }

    pub fn add_registry(&mut self, config: RegistryAuthConfig) {
        self.registries.push(config);
    }

    fn find_credentials(&self, host: &str) -> Option<&RegistryAuthConfig> {
        self.registries.iter().find(|r| {
            r.host == host ||
            host.contains(&r.host) ||
            // Handle docker.io specially
            (r.host == "docker.io" && (host == "registry-1.docker.io" || host == "index.docker.io"))
        })
    }
}

#[tonic::async_trait]
impl Auth for AuthServer {
    async fn credentials(
        &self,
        request: Request<CredentialsRequest>,
    ) -> Result<Response<CredentialsResponse>, Status> {
        let req = request.into_inner();
        tracing::debug!("Credentials requested for host: {}", req.host);

        if let Some(config) = self.find_credentials(&req.host) {
            tracing::debug!("Found credentials for host: {}", req.host);
            Ok(Response::new(CredentialsResponse {
                username: config.username.clone(),
                secret: config.password.clone(),
            }))
        } else {
            tracing::debug!("No credentials found for host: {}", req.host);
            // Return empty credentials (anonymous access)
            Ok(Response::new(CredentialsResponse {
                username: String::new(),
                secret: String::new(),
            }))
        }
    }

    async fn fetch_token(
        &self,
        request: Request<FetchTokenRequest>,
    ) -> Result<Response<FetchTokenResponse>, Status> {
        let req = request.into_inner();
        tracing::debug!(
            "FetchToken requested - Host: {}, Realm: {}, Service: {}, Scopes: {:?}",
            req.host, req.realm, req.service, req.scopes
        );

        // For most cases, BuildKit will handle token exchange
        // We just need to provide basic auth credentials via the Credentials RPC
        Ok(Response::new(FetchTokenResponse {
            token: String::new(),
            expires_in: 0,
            issued_at: 0,
        }))
    }

    async fn get_token_authority(
        &self,
        _request: Request<GetTokenAuthorityRequest>,
    ) -> Result<Response<GetTokenAuthorityResponse>, Status> {
        // Not implementing token authority for now
        Ok(Response::new(GetTokenAuthorityResponse {
            public_key: vec![],
        }))
    }

    async fn verify_token_authority(
        &self,
        _request: Request<VerifyTokenAuthorityRequest>,
    ) -> Result<Response<VerifyTokenAuthorityResponse>, Status> {
        // Not implementing token authority for now
        Ok(Response::new(VerifyTokenAuthorityResponse {
            signed: vec![],
        }))
    }
}
