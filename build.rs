use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    // Configure tonic-build
    tonic_build::configure()
        .build_server(false) // We only need the client
        .build_client(true)
        .out_dir(&out_dir)
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::prost_types")
        .compile_protos(
            &[
                "proto/api/services/control/control.proto",
            ],
            &["proto"], // Include path
        )?;

    println!("cargo:rerun-if-changed=proto/");

    Ok(())
}
