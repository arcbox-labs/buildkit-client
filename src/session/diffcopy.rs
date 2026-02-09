//! DiffCopy Protocol Implementation
//!
//! This module implements BuildKit's DiffCopy file synchronization protocol.
//! DiffCopy is a bidirectional streaming protocol that runs inside the HTTP/2-over-gRPC
//! tunnel and is tightly coupled with BuildKit's session architecture.
//!
//! ## Protocol Overview
//!
//! DiffCopy follows this flow:
//! 1. Client sends headers (dir_name, followpaths)
//! 2. Server sends STAT packets for all files/dirs (depth-first, sorted alphabetically)
//! 3. Server sends empty STAT packet to signal end of listing
//! 4. Client sends REQ packets for files it needs
//! 5. Server sends DATA packets with file contents
//! 6. Client sends FIN when done requesting
//! 7. Server sends FIN to acknowledge completion
//!
//! ## Key Requirements
//!
//! - Files must be sent in **depth-first order**
//! - Within each directory, entries must be **sorted alphabetically**
//! - Directory sizes must be 0 (fsutil protocol requirement)
//! - File modes must be in Go FileMode format (use `filemode` crate)
//!
//! ## References
//!
//! - BuildKit source: `github.com/moby/buildkit/session/filesync`
//! - fsutil reference: `github.com/tonistiigi/fsutil` (send.go, receive.go)

use crate::error::{Error, Result};
use crate::proto::fsutil::types::{packet::PacketType, Packet, Stat};
use bytes::Bytes;
use filemode::{GoFileMode, UnixMode};
use h2::server::SendResponse;
use http::{Response, StatusCode};
use prost::Message as ProstMessage;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::io::AsyncReadExt;

use super::FileSyncServer;

/// Handle a DiffCopy streaming request from BuildKit
///
/// This is the main entry point for the DiffCopy protocol. It handles the complete
/// bidirectional streaming session including sending file metadata and responding
/// to file data requests.
pub(super) async fn handle_diff_copy_stream(
    file_sync: &FileSyncServer,
    mut request_stream: h2::RecvStream,
    mut respond: SendResponse<Bytes>,
    dir_name: Option<String>,
    followpaths: Vec<String>,
) -> Result<()> {
    static CALL_COUNTER: AtomicU32 = AtomicU32::new(0);
    let call_id = CALL_COUNTER.fetch_add(1, Ordering::SeqCst);

    tracing::info!(
        "handle_diff_copy_stream called (call #{}, dir_name: {:?}, followpaths: {:?})",
        call_id,
        dir_name,
        followpaths
    );
    eprintln!(
        "\n========== DiffCopy Call #{} (dir_name: {:?}, followpaths: {:?}) ==========",
        call_id, dir_name, followpaths
    );

    tracing::info!("FileSync.DiffCopy streaming started (call #{})", call_id);

    // Build response headers
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/grpc")
        .body(())
        .unwrap();

    let mut send_stream = respond
        .send_response(response, false)
        .map_err(|e| Error::Http2Stream { source: e })?;

    tracing::info!("Sent response headers for DiffCopy");

    // Get the root path from FileSyncServer
    let root_path = file_sync.get_root_path();
    tracing::info!(
        "Starting to send STAT packets from: {} (call #{})",
        root_path.display(),
        call_id
    );
    eprintln!(
        "Root path: {}, is_dir: {}",
        root_path.display(),
        root_path.is_dir()
    );

    // Determine what to send based on dir_name header
    let mut file_map = HashMap::new();
    let mut id_counter = 0u32;

    let send_only_dockerfile = dir_name.as_deref() == Some("dockerfile");

    if send_only_dockerfile {
        // BuildKit only wants the Dockerfile
        send_dockerfile_only(&root_path, &followpaths, &mut send_stream, &mut file_map).await?;
    } else {
        // BuildKit wants the full context
        send_full_context(
            &root_path,
            &followpaths,
            &mut send_stream,
            &mut file_map,
            &mut id_counter,
        )
        .await?;
    }

    // Send final empty STAT packet to indicate end of stats
    let final_stat_packet = Packet {
        r#type: PacketType::PacketStat as i32,
        stat: None,
        id: 0,
        data: vec![],
    };
    send_grpc_packet(&mut send_stream, &final_stat_packet).await?;

    tracing::info!("Sent all STAT packets (including final empty STAT), now waiting for REQ packets from BuildKit");

    // Process REQ packets from BuildKit
    process_file_requests(&mut request_stream, &mut send_stream, &file_map).await?;

    tracing::info!("DiffCopy completed, sending FIN packet");

    // Send FIN packet to indicate all transfers are complete
    let fin_packet = Packet {
        r#type: PacketType::PacketFin as i32,
        stat: None,
        id: 0,
        data: vec![],
    };

    send_grpc_packet(&mut send_stream, &fin_packet).await?;
    tracing::debug!("Sent final FIN packet");

    // Send success trailers
    let trailers = Response::builder()
        .header("grpc-status", "0")
        .body(())
        .unwrap();

    send_stream
        .send_trailers(trailers.headers().clone())
        .map_err(|e| Error::Http2Stream { source: e })?;

    Ok(())
}

/// Send only the Dockerfile (when dir_name="dockerfile")
async fn send_dockerfile_only(
    root_path: &Path,
    followpaths: &[String],
    send_stream: &mut h2::SendStream<Bytes>,
    file_map: &mut HashMap<u32, PathBuf>,
) -> Result<()> {
    let dockerfile_name = if !followpaths.is_empty() && followpaths[0].ends_with(".Dockerfile") {
        followpaths[0].clone()
    } else {
        "Dockerfile".to_string()
    };

    eprintln!(
        "BuildKit requested 'dockerfile' - sending only {}",
        dockerfile_name
    );

    let dockerfile_path = root_path.join(&dockerfile_name);
    if !dockerfile_path.exists() {
        tracing::error!(
            "{} not found at {}",
            dockerfile_name,
            dockerfile_path.display()
        );
        return Err(Error::PathNotFound(dockerfile_path));
    }

    let metadata = tokio::fs::metadata(&dockerfile_path).await?;

    let mut stat = Stat {
        path: dockerfile_name.clone(),
        mode: 0,
        uid: 0,
        gid: 0,
        size: metadata.len() as i64,
        mod_time: 0,
        linkname: String::new(),
        devmajor: 0,
        devminor: 0,
        xattrs: HashMap::new(),
    };

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let unix_mode = metadata.permissions().mode();
        stat.mode = GoFileMode::from(UnixMode::from(unix_mode)).as_u32();
    }

    #[cfg(not(unix))]
    {
        stat.mode = 0o644; // Regular file in Go FileMode format
    }

    let mode = stat.mode;
    let stat_packet = Packet {
        r#type: PacketType::PacketStat as i32,
        stat: Some(stat),
        id: 0,
        data: vec![],
    };

    eprintln!(
        "DFS: Sending STAT #0: {} (FILE, mode: 0o{:o})",
        dockerfile_name, mode
    );
    send_grpc_packet(send_stream, &stat_packet).await?;

    file_map.insert(0, dockerfile_path);
    Ok(())
}

/// Send full directory tree using depth-first traversal
async fn send_full_context(
    root_path: &Path,
    followpaths: &[String],
    send_stream: &mut h2::SendStream<Bytes>,
    file_map: &mut HashMap<u32, PathBuf>,
    id_counter: &mut u32,
) -> Result<()> {
    if followpaths.is_empty() {
        eprintln!("BuildKit requested full context - sending entire directory tree");
    } else {
        eprintln!(
            "BuildKit requested filtered context - followpaths: {:?}",
            followpaths
        );
    }

    send_stat_packets_dfs(
        root_path.to_path_buf(),
        String::new(),
        send_stream,
        file_map,
        id_counter,
        if followpaths.is_empty() {
            None
        } else {
            Some(followpaths)
        },
    )
    .await
}

/// Send STAT packets using depth-first traversal
///
/// This function recursively traverses the directory tree in depth-first order,
/// sending STAT packets for each file and directory. This is the correct way to
/// send files to BuildKit's fsutil validator, which requires files in depth-first
/// order with entries sorted alphabetically within each directory.
///
/// If `followpaths` is Some, only sends files in the list and their parent directories.
fn send_stat_packets_dfs<'a>(
    path: PathBuf,
    prefix: String,
    stream: &'a mut h2::SendStream<Bytes>,
    file_map: &'a mut HashMap<u32, PathBuf>,
    id_counter: &'a mut u32,
    followpaths: Option<&'a [String]>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        tracing::debug!(
            "send_stat_packets_dfs: {} (prefix: {}, followpaths: {:?})",
            path.display(),
            prefix,
            followpaths
        );

        // Build set of paths to include if followpaths is specified
        let include_paths = if let Some(paths) = followpaths {
            let mut set = HashSet::new();
            for p in paths {
                set.insert(p.clone());
                // Add all parent directories
                let mut parent = p.as_str();
                while let Some(idx) = parent.rfind('/') {
                    parent = &parent[..idx];
                    set.insert(parent.to_string());
                }
            }
            tracing::debug!(
                "Built include_paths set with {} entries: {:?}",
                set.len(),
                set
            );
            Some(set)
        } else {
            None
        };

        // Read all entries in this directory
        let mut entries = Vec::new();
        let mut dir_entries = tokio::fs::read_dir(&path).await?;

        while let Some(entry) = dir_entries.next_entry().await? {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy().to_string();
            let entry_path = entry.path();
            let metadata = entry.metadata().await?;

            entries.push((name, entry_path, metadata));
        }

        // Sort entries alphabetically by name (fsutil requirement)
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Process entries in sorted order (depth-first)
        for (name, entry_path, metadata) in entries {
            let rel_path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            // Skip if not in include_paths (when filtering is enabled)
            if let Some(ref paths) = include_paths {
                if !paths.contains(&rel_path) {
                    tracing::debug!("Skipping {} (not in followpaths)", rel_path);
                    eprintln!("DFS: Skipping {} (not in include_paths)", rel_path);
                    continue;
                } else {
                    eprintln!("DFS: Including {} (found in include_paths)", rel_path);
                }
            }

            let entry_id = *id_counter;
            *id_counter += 1;

            // Create and send STAT packet for this entry
            let mut stat = Stat {
                path: rel_path.clone(),
                mode: 0,
                uid: 0,
                gid: 0,
                // For directories, size must be 0 (fsutil protocol requirement)
                size: if metadata.is_dir() {
                    0
                } else {
                    metadata.len() as i64
                },
                mod_time: 0,
                linkname: String::new(),
                devmajor: 0,
                devminor: 0,
                xattrs: HashMap::new(),
            };

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let unix_mode = metadata.permissions().mode();
                stat.mode = GoFileMode::from(UnixMode::from(unix_mode)).as_u32();
            }

            #[cfg(not(unix))]
            {
                // On non-Unix platforms, construct mode in Go FileMode format directly
                stat.mode = if metadata.is_dir() {
                    0x80000000 | 0o755 // GO_MODE_DIR | 0o755
                } else {
                    0o644 // Just permissions for regular files
                };
            }

            let mode = stat.mode;
            let size = stat.size;
            let path_sent = stat.path.clone();
            let stat_packet = Packet {
                r#type: PacketType::PacketStat as i32,
                stat: Some(stat),
                id: entry_id,
                data: vec![],
            };

            tracing::info!(
                "Sending STAT packet for: {} (id: {}, mode: 0o{:o})",
                path_sent,
                entry_id,
                mode
            );
            eprintln!(
                "DFS: Sending STAT #{}: {} ({}, mode: 0o{:o} / 0x{:x}, size: {}, is_dir: {})",
                entry_id,
                path_sent,
                if metadata.is_dir() { "DIR" } else { "FILE" },
                mode,
                mode,
                size,
                (mode & 0o040000) != 0
            );
            send_grpc_packet(stream, &stat_packet).await?;

            // Store file path in map for later data requests (only for files)
            if metadata.is_file() {
                file_map.insert(entry_id, entry_path.clone());
            }

            // Recursively process directories
            if metadata.is_dir() {
                send_stat_packets_dfs(
                    entry_path,
                    rel_path,
                    stream,
                    file_map,
                    id_counter,
                    followpaths,
                )
                .await?;
            }
        }

        Ok(())
    })
}

/// Process incoming REQ packets from BuildKit and send file data
async fn process_file_requests(
    request_stream: &mut h2::RecvStream,
    send_stream: &mut h2::SendStream<Bytes>,
    file_map: &HashMap<u32, PathBuf>,
) -> Result<()> {
    let mut buffer = Vec::new();
    let mut received_fin = false;

    loop {
        match request_stream.data().await {
            Some(Ok(chunk)) => {
                buffer.extend_from_slice(&chunk);
                let _ = request_stream.flow_control().release_capacity(chunk.len());

                // Try to parse complete gRPC messages from buffer
                while buffer.len() >= 5 {
                    // Read gRPC frame header (5 bytes)
                    let compressed = buffer[0];
                    let length =
                        u32::from_be_bytes([buffer[1], buffer[2], buffer[3], buffer[4]]) as usize;

                    if buffer.len() < 5 + length {
                        break; // Not enough data yet
                    }

                    // Extract the complete message
                    let message_data = buffer[5..5 + length].to_vec();
                    buffer.drain(0..5 + length);

                    if compressed != 0 {
                        tracing::warn!("Received compressed message, skipping");
                        continue;
                    }

                    // Decode the packet
                    let packet = match Packet::decode(Bytes::from(message_data)) {
                        Ok(p) => p,
                        Err(e) => {
                            tracing::error!("Failed to decode packet: {}", e);
                            continue;
                        }
                    };

                    let packet_type =
                        PacketType::try_from(packet.r#type).unwrap_or(PacketType::PacketStat);
                    tracing::debug!(
                        "Received packet type: {:?}, id: {}, has_stat: {}",
                        packet_type,
                        packet.id,
                        packet.stat.is_some()
                    );

                    match packet_type {
                        PacketType::PacketReq => {
                            tracing::info!("Received REQ packet with id: {}", packet.id);

                            if let Some(file_path) = file_map.get(&packet.id) {
                                tracing::info!(
                                    "Sending file data for id {}: {}",
                                    packet.id,
                                    file_path.display()
                                );
                                send_file_data_packets(file_path.clone(), packet.id, send_stream)
                                    .await?;
                            } else {
                                tracing::warn!(
                                    "File ID {} not found in map (probably a directory, ignoring)",
                                    packet.id
                                );
                            }
                        }
                        PacketType::PacketFin => {
                            tracing::info!("Received FIN packet from BuildKit, ending transfer");
                            received_fin = true;
                            break;
                        }
                        _ => {
                            tracing::debug!("Ignoring packet type: {:?}", packet_type);
                        }
                    }
                }

                if received_fin {
                    break;
                }
            }
            Some(Err(e)) => {
                tracing::error!("Error reading request stream: {}", e);
                break;
            }
            None => {
                tracing::info!("Request stream ended");
                break;
            }
        }
    }

    Ok(())
}

/// Send file data as DATA packets in response to a REQ
async fn send_file_data_packets(
    path: PathBuf,
    req_id: u32,
    stream: &mut h2::SendStream<Bytes>,
) -> Result<()> {
    tracing::info!("Sending file data for: {} (id: {})", path.display(), req_id);

    let mut file = tokio::fs::File::open(&path).await?;
    let mut buffer = vec![0u8; 32 * 1024]; // 32KB chunks

    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }

        let data_packet = Packet {
            r#type: PacketType::PacketData as i32,
            stat: None,
            id: req_id,
            data: buffer[..n].to_vec(),
        };

        send_grpc_packet(stream, &data_packet).await?;
    }

    // Send empty DATA packet to indicate end of this file
    let eof_packet = Packet {
        r#type: PacketType::PacketData as i32,
        stat: None,
        id: req_id,
        data: vec![],
    };

    send_grpc_packet(stream, &eof_packet).await?;
    tracing::debug!("Sent EOF (empty DATA) packet for id: {}", req_id);

    Ok(())
}

/// Send a single gRPC-framed packet over the h2 stream
async fn send_grpc_packet(stream: &mut h2::SendStream<Bytes>, packet: &Packet) -> Result<()> {
    let mut payload = Vec::new();
    packet.encode(&mut payload)?;

    // Add gRPC framing (5-byte prefix)
    let mut framed = Vec::new();
    framed.push(0); // No compression
    framed.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    framed.extend_from_slice(&payload);

    let packet_type = PacketType::try_from(packet.r#type).ok();
    tracing::trace!(
        "Sending packet: type={:?}, id={}, data_len={}, total_frame_len={}",
        packet_type,
        packet.id,
        packet.data.len(),
        framed.len()
    );

    stream
        .send_data(Bytes::from(framed), false)
        .map_err(|e| Error::Http2Stream { source: e })?;

    // Give the h2 stream a chance to flush
    tokio::task::yield_now().await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use http::Request;
    use std::future::Future;
    use std::pin::Pin;
    use tokio::io::duplex;
    use tokio::sync::oneshot;

    type BoxResultFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>;

    /// Helper to drive an in-memory h2 server, execute `server_logic`, and capture emitted packets.
    async fn capture_packets<T, F>(server_logic: F) -> (Vec<Packet>, T)
    where
        F: for<'a> FnOnce(&'a mut h2::SendStream<Bytes>) -> BoxResultFuture<'a, T> + Send + 'static,
        T: Send + 'static,
    {
        let (client_io, server_io) = duplex(256 * 1024);
        let (result_tx, result_rx) = oneshot::channel();
        let (done_tx, done_rx) = oneshot::channel();

        let server_task = tokio::spawn(async move {
            let mut connection = h2::server::handshake(server_io)
                .await
                .expect("server handshake");

            if let Some(result) = connection.accept().await {
                let (_request, mut respond) = result.expect("server accept");
                let response = Response::builder().status(StatusCode::OK).body(()).unwrap();
                let mut send_stream = respond
                    .send_response(response, false)
                    .expect("send response");

                // Spawn a task to keep driving the server h2 connection
                // so that send_data() calls actually flush to the wire.
                let drive =
                    tokio::spawn(async move { while connection.accept().await.is_some() {} });

                let outcome = server_logic(&mut send_stream).await;
                let _ = send_stream.send_data(Bytes::new(), true);
                let _ = result_tx.send(outcome);
                let _ = done_rx.await;
                drive.abort();
            } else {
                let _ = result_tx.send(Err(Error::other("client did not open request in test")));
            }
        });

        let (client, connection) = h2::client::handshake(client_io)
            .await
            .expect("client handshake");
        let client_task = tokio::spawn(async move {
            let _ = connection.await;
        });

        let (response_future, _send_stream) = client
            .clone()
            .ready()
            .await
            .expect("client ready")
            .send_request(Request::builder().uri("/").body(()).unwrap(), true)
            .expect("send request");
        drop(client);

        let response = response_future.await.expect("await response");

        let mut body = response.into_body();
        let mut buffer = BytesMut::new();
        while let Some(chunk) = body.data().await {
            let chunk = chunk.expect("body chunk");
            // Release flow control capacity so the server can send more data.
            let _ = body.flow_control().release_capacity(chunk.len());
            buffer.extend_from_slice(&chunk);
        }

        let outcome = result_rx
            .await
            .expect("receive outcome")
            .expect("server logic");
        let packets = decode_packets(&buffer);

        let _ = done_tx.send(());

        server_task.await.expect("server task");
        client_task.abort();

        (packets, outcome)
    }

    fn decode_packets(buffer: &BytesMut) -> Vec<Packet> {
        let mut packets = Vec::new();
        let mut slice: &[u8] = buffer.as_ref();

        while !slice.is_empty() {
            assert!(
                slice.len() >= 5,
                "incomplete gRPC frame encountered ({} bytes remain)",
                slice.len()
            );

            assert_eq!(slice[0], 0, "expected uncompressed gRPC frame");
            let length = u32::from_be_bytes([slice[1], slice[2], slice[3], slice[4]]) as usize;
            let total_len = 5 + length;
            assert!(
                slice.len() >= total_len,
                "gRPC frame length {} exceeds remaining buffer {}",
                length,
                slice.len()
            );

            let packet = Packet::decode(&slice[5..total_len]).expect("decode packet");
            packets.push(packet);

            slice = &slice[total_len..];
        }

        packets
    }

    fn create_test_context(root: &Path) {
        std::fs::write(root.join("Dockerfile"), "FROM alpine\n").unwrap();

        let app_dir = root.join("app");
        std::fs::create_dir_all(&app_dir).unwrap();
        std::fs::write(app_dir.join("config.txt"), "config").unwrap();
        std::fs::write(app_dir.join("main.txt"), "main").unwrap();

        let sub_dir = app_dir.join("subdir");
        std::fs::create_dir_all(&sub_dir).unwrap();
        std::fs::write(sub_dir.join("data.txt"), "data").unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn stat_packets_follow_depth_first_order() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        create_test_context(&root_path);

        let root_for_closure = root_path.clone();
        let (packets, (file_map, id_counter)) = capture_packets(move |send_stream| {
            let root = root_for_closure.clone();
            Box::pin(async move {
                let mut file_map = HashMap::new();
                let mut counter = 0u32;

                send_stat_packets_dfs(
                    root,
                    String::new(),
                    send_stream,
                    &mut file_map,
                    &mut counter,
                    None,
                )
                .await?;

                Ok((file_map, counter))
            })
        })
        .await;

        let paths: Vec<String> = packets
            .iter()
            .map(|packet| {
                assert_eq!(
                    PacketType::try_from(packet.r#type).unwrap(),
                    PacketType::PacketStat
                );
                packet.stat.as_ref().unwrap().path.clone()
            })
            .collect();

        assert_eq!(
            paths,
            vec![
                "Dockerfile",
                "app",
                "app/config.txt",
                "app/main.txt",
                "app/subdir",
                "app/subdir/data.txt"
            ]
        );

        assert_eq!(id_counter, 6);
        assert_eq!(file_map.len(), 4);
        assert_eq!(file_map.get(&0).unwrap(), &root_path.join("Dockerfile"));
        assert_eq!(file_map.get(&2).unwrap(), &root_path.join("app/config.txt"));
        assert_eq!(file_map.get(&3).unwrap(), &root_path.join("app/main.txt"));
        assert_eq!(
            file_map.get(&5).unwrap(),
            &root_path.join("app/subdir/data.txt")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn stat_packets_respect_followpaths_filter() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path().to_path_buf();
        create_test_context(&root_path);

        let followpaths = vec!["app/subdir/data.txt".to_string()];
        let root_for_closure = root_path.clone();
        let follow_for_closure = followpaths.clone();

        let (packets, (file_map, id_counter)) = capture_packets(move |send_stream| {
            let root = root_for_closure.clone();
            let follow = follow_for_closure.clone();
            Box::pin(async move {
                let mut file_map = HashMap::new();
                let mut counter = 0u32;

                send_stat_packets_dfs(
                    root,
                    String::new(),
                    send_stream,
                    &mut file_map,
                    &mut counter,
                    Some(&follow),
                )
                .await?;

                Ok((file_map, counter))
            })
        })
        .await;

        let paths: Vec<String> = packets
            .iter()
            .map(|packet| {
                assert_eq!(
                    PacketType::try_from(packet.r#type).unwrap(),
                    PacketType::PacketStat
                );
                packet.stat.as_ref().unwrap().path.clone()
            })
            .collect();

        assert_eq!(paths, vec!["app", "app/subdir", "app/subdir/data.txt"]);

        assert_eq!(id_counter, 3);
        assert_eq!(file_map.len(), 1);
        assert_eq!(
            file_map.get(&2).unwrap(),
            &root_path.join("app/subdir/data.txt")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn file_data_packets_stream_contents_and_eof() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root_path = temp_dir.path();
        let file_path = root_path.join("large.txt");

        let content_len = 70 * 1024 + 7;
        let expected_content = vec![b'a'; content_len];
        std::fs::write(&file_path, &expected_content).unwrap();

        let req_id = 42u32;
        let file_for_closure = file_path.clone();

        let (packets, ()) = capture_packets(move |send_stream| {
            let path = file_for_closure.clone();
            Box::pin(async move { send_file_data_packets(path, req_id, send_stream).await })
        })
        .await;

        let mut offset = 0usize;
        let mut expected_sizes = Vec::new();
        let mut remaining = expected_content.len();
        while remaining > 0 {
            let chunk = remaining.min(32 * 1024);
            expected_sizes.push(chunk);
            remaining -= chunk;
        }
        expected_sizes.push(0);

        assert_eq!(packets.len(), expected_sizes.len());
        for (packet, expected_size) in packets.iter().zip(expected_sizes.iter()) {
            assert_eq!(
                PacketType::try_from(packet.r#type).unwrap(),
                PacketType::PacketData
            );
            assert_eq!(packet.id, req_id);

            let data = &packet.data;
            assert_eq!(data.len(), *expected_size);

            if *expected_size > 0 {
                let end = offset + *expected_size;
                assert_eq!(
                    data,
                    &expected_content[offset..end],
                    "mismatched chunk contents"
                );
                offset = end;
            }
        }

        assert_eq!(offset, expected_content.len());
    }
}
