//! gRPC tunneling protocol for session stream using h2
//!
//! BuildKit establishes an HTTP/2 connection inside the bidirectional session stream.
//! We use the h2 crate to handle the HTTP/2 server protocol.

use crate::error::{Error, Result};
use bytes::Bytes;
use h2::server::{self, SendResponse};
use http::{Request, Response, StatusCode};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context as TaskContext, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::sync::mpsc;
use prost::Message as ProstMessage;

use crate::proto::moby::buildkit::v1::BytesMessage;
use super::{FileSyncServer, AuthServer, SecretsServer};

/// Stream multiplexer for handling gRPC tunneled through session
pub struct GrpcTunnel {
    file_sync: Option<FileSyncServer>,
    auth: Option<AuthServer>,
    secrets: Option<SecretsServer>,
}

impl GrpcTunnel {
    /// Create a new gRPC tunnel
    pub fn new(
        _response_tx: mpsc::Sender<BytesMessage>,
        file_sync: Option<FileSyncServer>,
        auth: Option<AuthServer>,
        secrets: Option<SecretsServer>,
    ) -> Self {
        Self {
            file_sync,
            auth,
            secrets,
        }
    }

    /// Start HTTP/2 server over the session stream
    pub async fn serve(
        self,
        inbound_rx: mpsc::Receiver<BytesMessage>,
        outbound_tx: mpsc::Sender<BytesMessage>,
    ) -> Result<()> {
        let tunnel = Arc::new(self);

        // Create a wrapper that implements AsyncRead + AsyncWrite
        let stream = MessageStream::new(inbound_rx, outbound_tx);

        // Start HTTP/2 server
        let mut h2_conn = server::handshake(stream).await
            .map_err(|e| Error::Http2Handshake { source: e })?;

        tracing::info!("HTTP/2 server started in session tunnel");

        // Accept incoming HTTP/2 streams
        while let Some(result) = h2_conn.accept().await {
            let (request, respond) = result.map_err(|e| Error::Http2Stream { source: e })?;
            let tunnel_ref = Arc::clone(&tunnel);

            tokio::spawn(async move {
                if let Err(e) = tunnel_ref.handle_request(request, respond).await {
                    tracing::error!("Failed to handle gRPC request: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single gRPC request
    async fn handle_request(
        &self,
        req: Request<h2::RecvStream>,
        respond: SendResponse<Bytes>,
    ) -> Result<()> {
        let method = req.uri().path().to_string();
        tracing::info!("Received gRPC call: {}", method);

        // Debug: print all request headers
        eprintln!("\n=== Request Headers for {} ===", method);
        for (name, value) in req.headers() {
            if let Ok(v) = value.to_str() {
                eprintln!("  {}: {}", name, v);
            }
        }

        // Extract dir-name header before consuming req
        let dir_name = req.headers()
            .get("dir-name")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Extract followpaths header (can have multiple values)
        let followpaths: Vec<String> = req.headers()
            .get_all("followpaths")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        let body = req.into_body();

        // Dispatch to appropriate service
        match method.as_str() {
            "/grpc.health.v1.Health/Check" => {
                // Read request body for unary RPC
                let payload = Self::read_unary_request(body).await?;
                let response_payload = self.handle_health_check(payload).await?;
                self.send_success_response(respond, response_payload).await
            }
            "/moby.filesync.v1.FileSync/DiffCopy" => {
                // DiffCopy is a bidirectional streaming RPC - delegate to diffcopy module
                let file_sync = match &self.file_sync {
                    Some(fs) => fs,
                    None => {
                        tracing::error!("FileSync not available");
                        return self.send_error_response(respond, "FileSync not available").await;
                    }
                };
                super::diffcopy::handle_diff_copy_stream(file_sync, body, respond, dir_name, followpaths).await
            }
            "/moby.filesync.v1.Auth/GetTokenAuthority" => {
                // Token-based auth not supported - return error to make BuildKit fall back
                // BuildKit requires either a valid pubkey or error to properly fallback to Credentials
                tracing::info!("Auth.GetTokenAuthority called - returning not implemented");
                self.send_error_response(respond, "Token auth not implemented").await
            }
            "/moby.filesync.v1.Auth/Credentials" => {
                let payload = Self::read_unary_request(body).await?;
                let response_payload = self.handle_auth_credentials(payload).await?;
                self.send_success_response(respond, response_payload).await
            }
            "/moby.filesync.v1.Auth/FetchToken" => {
                let payload = Self::read_unary_request(body).await?;
                let response_payload = self.handle_auth_fetch_token(payload).await?;
                self.send_success_response(respond, response_payload).await
            }
            "/moby.buildkit.secrets.v1.Secrets/GetSecret" => {
                let payload = Self::read_unary_request(body).await?;
                let response_payload = self.handle_secrets_get_secret(payload).await?;
                self.send_success_response(respond, response_payload).await
            }
            _ => {
                tracing::warn!("Unknown gRPC method: {}", method);
                self.send_error_response(respond, "Unimplemented").await
            }
        }
    }

    /// Read complete request body for unary RPC
    async fn read_unary_request(mut body: h2::RecvStream) -> Result<Bytes> {
        let mut request_data = Vec::new();

        while let Some(chunk) = body.data().await {
            let chunk = chunk.map_err(|e| Error::Http2Stream { source: e })?;
            request_data.extend_from_slice(&chunk);
            let _ = body.flow_control().release_capacity(chunk.len());
        }

        // Skip the 5-byte gRPC prefix (1 byte compression + 4 bytes length)
        let payload = if request_data.len() > 5 {
            Bytes::copy_from_slice(&request_data[5..])
        } else {
            Bytes::new()
        };

        Ok(payload)
    }

    /// Send successful gRPC response
    async fn send_success_response(
        &self,
        mut respond: SendResponse<Bytes>,
        payload: Bytes,
    ) -> Result<()> {
        // Build gRPC response headers (without grpc-status - that goes in trailers)
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/grpc")
            .body(())
            .unwrap();

        let mut send_stream = respond.send_response(response, false)
            .map_err(|e| Error::Http2Stream { source: e })?;

        // Send response with gRPC framing (5-byte prefix)
        let mut framed = Vec::new();
        framed.push(0); // No compression
        framed.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        framed.extend_from_slice(&payload);

        send_stream.send_data(Bytes::from(framed), false)
            .map_err(|e| Error::Http2Stream { source: e })?;

        // Send trailers with grpc-status
        let trailers = Response::builder()
            .header("grpc-status", "0")
            .body(())
            .unwrap();

        send_stream.send_trailers(trailers.headers().clone())
            .map_err(|e| Error::Http2Stream { source: e })?;

        Ok(())
    }

    /// Send error gRPC response
    async fn send_error_response(
        &self,
        mut respond: SendResponse<Bytes>,
        message: &str,
    ) -> Result<()> {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/grpc")
            .header("grpc-status", "12") // UNIMPLEMENTED
            .header("grpc-message", message)
            .body(())
            .unwrap();

        respond.send_response(response, true)
            .map_err(|e| Error::Http2Stream { source: e })?;

        Ok(())
    }


    /// Handle Auth.GetTokenAuthority request
    #[allow(dead_code)]
    async fn handle_auth_get_token_authority(&self, payload: Bytes) -> Result<Bytes> {
        use crate::proto::moby::filesync::v1::{GetTokenAuthorityRequest, GetTokenAuthorityResponse};

        let request = GetTokenAuthorityRequest::decode(payload)
            .map_err(|e| Error::decode("GetTokenAuthorityRequest", e))?;

        tracing::info!("Auth.GetTokenAuthority request for host: {}", request.host);

        // Return empty response - we don't implement token-based auth
        // BuildKit will detect empty public_key and fall back to Credentials method
        let response = GetTokenAuthorityResponse {
            public_key: vec![],
        };

        let mut buf = Vec::new();
        response.encode(&mut buf)?;
        Ok(Bytes::from(buf))
    }

    /// Handle Auth.Credentials request
    async fn handle_auth_credentials(&self, payload: Bytes) -> Result<Bytes> {
        use crate::proto::moby::filesync::v1::CredentialsRequest;
        use tonic::Request;
        use crate::proto::moby::filesync::v1::auth_server::Auth;

        let request = CredentialsRequest::decode(payload)
            .map_err(|e| Error::decode("CredentialsRequest", e))?;

        tracing::info!("Auth.Credentials request for host: {}", request.host);

        // Use AuthServer if configured, otherwise return empty credentials
        let response = if let Some(auth) = &self.auth {
            match auth.credentials(Request::new(request.clone())).await {
                Ok(resp) => {
                    let inner = resp.into_inner();
                    if !inner.username.is_empty() {
                        tracing::debug!("Returning credentials for host: {} (username: {})",
                            request.host, inner.username);
                    } else {
                        tracing::debug!("No credentials found for host: {}, returning empty", request.host);
                    }
                    inner
                }
                Err(status) => {
                    tracing::warn!("Failed to get credentials: {}, returning empty", status.message());
                    use crate::proto::moby::filesync::v1::CredentialsResponse;
                    CredentialsResponse {
                        username: String::new(),
                        secret: String::new(),
                    }
                }
            }
        } else {
            tracing::debug!("No auth configured, returning empty credentials");
            use crate::proto::moby::filesync::v1::CredentialsResponse;
            CredentialsResponse {
                username: String::new(),
                secret: String::new(),
            }
        };

        let mut buf = Vec::new();
        response.encode(&mut buf)?;
        Ok(Bytes::from(buf))
    }

    /// Handle Auth.FetchToken request
    async fn handle_auth_fetch_token(&self, _payload: Bytes) -> Result<Bytes> {
        use crate::proto::moby::filesync::v1::FetchTokenResponse;

        tracing::info!("Auth.FetchToken called");

        let response = FetchTokenResponse {
            token: String::new(),
            expires_in: 0,
            issued_at: 0,
        };

        let mut buf = Vec::new();
        response.encode(&mut buf)?;
        Ok(Bytes::from(buf))
    }

    /// Handle Secrets.GetSecret request
    async fn handle_secrets_get_secret(&self, payload: Bytes) -> Result<Bytes> {
        use crate::proto::moby::secrets::v1::GetSecretRequest;

        let request = GetSecretRequest::decode(payload)
            .map_err(|e| Error::decode("GetSecretRequest", e))?;

        tracing::info!("Secrets.GetSecret request for ID: {}", request.id);

        // If secrets service is not configured, return empty data
        let response = if let Some(secrets) = &self.secrets {
            // Use the SecretsServer's get_secret implementation through the Secrets trait
            use tonic::Request;
            use crate::proto::moby::secrets::v1::secrets_server::Secrets;

            match secrets.get_secret(Request::new(request.clone())).await {
                Ok(resp) => {
                    let inner = resp.into_inner();
                    tracing::debug!("Returning secret '{}' ({} bytes)", request.id, inner.data.len());
                    inner
                }
                Err(status) => {
                    tracing::warn!("Secret '{}' not found: {}", request.id, status.message());
                    return Err(Error::SecretNotFound(status.message().to_string()));
                }
            }
        } else {
            tracing::warn!("Secrets service not configured");
            return Err(Error::SecretsNotConfigured);
        };

        let mut buf = Vec::new();
        response.encode(&mut buf)?;
        Ok(Bytes::from(buf))
    }

    /// Handle Health.Check request
    async fn handle_health_check(&self, _payload: Bytes) -> Result<Bytes> {
        tracing::info!("Health check called");

        // Health check response: status = SERVING (1)
        // The proto definition is:
        // message HealthCheckResponse {
        //   enum ServingStatus {
        //     UNKNOWN = 0;
        //     SERVING = 1;
        //     NOT_SERVING = 2;
        //   }
        //   ServingStatus status = 1;
        // }

        // Manually encode: field 1, varint type, value 1
        let response = vec![0x08, 0x01]; // field 1 (0x08 = 0001|000) = value 1
        Ok(Bytes::from(response))
    }
}

/// A stream that wraps BytesMessage channels to implement AsyncRead + AsyncWrite
struct MessageStream {
    inbound_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<BytesMessage>>>,
    outbound_tx: mpsc::Sender<BytesMessage>,
    read_buffer: Vec<u8>,
    read_pos: usize,
}

impl MessageStream {
    fn new(
        inbound_rx: mpsc::Receiver<BytesMessage>,
        outbound_tx: mpsc::Sender<BytesMessage>,
    ) -> Self {
        Self {
            inbound_rx: Arc::new(tokio::sync::Mutex::new(inbound_rx)),
            outbound_tx,
            read_buffer: Vec::new(),
            read_pos: 0,
        }
    }
}

impl AsyncRead for MessageStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        // If we have buffered data, return it
        if self.read_pos < self.read_buffer.len() {
            let remaining = &self.read_buffer[self.read_pos..];
            let to_copy = remaining.len().min(buf.remaining());
            buf.put_slice(&remaining[..to_copy]);
            self.read_pos += to_copy;

            // Clear buffer if fully consumed
            if self.read_pos >= self.read_buffer.len() {
                self.read_buffer.clear();
                self.read_pos = 0;
            }

            return Poll::Ready(Ok(()));
        }

        // Try to receive next message
        let inbound_rx = self.inbound_rx.clone();
        let mut rx = match inbound_rx.try_lock() {
            Ok(rx) => rx,
            Err(_) => return Poll::Pending,
        };

        match rx.poll_recv(cx) {
            Poll::Ready(Some(msg)) => {
                self.read_buffer = msg.data;
                self.read_pos = 0;

                let to_copy = self.read_buffer.len().min(buf.remaining());
                buf.put_slice(&self.read_buffer[..to_copy]);
                self.read_pos = to_copy;

                Poll::Ready(Ok(()))
            }
            Poll::Ready(None) => Poll::Ready(Ok(())), // EOF
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsyncWrite for MessageStream {
    fn poll_write(
        self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        let msg = BytesMessage {
            data: buf.to_vec(),
        };

        // Try to send immediately (non-blocking)
        match self.outbound_tx.try_send(msg) {
            Ok(()) => Poll::Ready(Ok(buf.len())),
            Err(mpsc::error::TrySendError::Full(_)) => {
                // Channel is full, would block
                Poll::Pending
            }
            Err(mpsc::error::TrySendError::Closed(_)) => {
                Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Channel closed",
                )))
            }
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        _cx: &mut TaskContext<'_>,
    ) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
