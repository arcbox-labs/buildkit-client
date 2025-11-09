use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Check if proto files exist, if not, run initialization script
    let proto_dir = Path::new("proto");
    let control_proto = proto_dir.join("github.com/moby/buildkit/api/services/control/control.proto");

    if !control_proto.exists() {
        eprintln!("Proto files not found. Running initialization script...");
        eprintln!("Please run: ./scripts/init-proto.sh");
        eprintln!("Or manually clone proto files from https://github.com/moby/buildkit");

        // Try to run the script automatically
        let script_path = Path::new("scripts/init-proto.sh");
        if script_path.exists() {
            let status = Command::new("bash")
                .arg(script_path)
                .status();

            match status {
                Ok(s) if s.success() => {
                    eprintln!("Proto files initialized successfully!");
                }
                _ => {
                    eprintln!("Failed to initialize proto files automatically.");
                    eprintln!("Please run: ./scripts/init-proto.sh manually");
                }
            }
        }
    }

    // Configure tonic-build
    tonic_build::configure()
        .build_server(true) // We need server for session services
        .build_client(true)
        .out_dir(&out_dir)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        .compile_protos(
            &[
                "proto/github.com/moby/buildkit/api/services/control/control.proto",
                "proto/github.com/moby/buildkit/session/filesync/filesync.proto",
                "proto/github.com/moby/buildkit/session/auth/auth.proto",
            ],
            &["proto"], // Include path
        )?;

    println!("cargo:rerun-if-changed=proto/github.com/moby/buildkit/api/");
    println!("cargo:rerun-if-changed=proto/github.com/moby/buildkit/session/");
    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
