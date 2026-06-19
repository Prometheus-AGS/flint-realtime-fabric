use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");

    let proto_root = workspace_root.join("proto");
    let protos: Vec<PathBuf> = [
        "flint/v1/envelope.proto",
        "flint/v1/entity.proto",
        "flint/v1/agent.proto",
        "flint/v1/signal.proto",
        "flint/v1/sync.proto",
        "flint/v1/authz.proto",
    ]
    .iter()
    .map(|p| proto_root.join(p))
    .collect();

    let includes = [proto_root.clone()];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&protos, &includes)?;

    println!("cargo:rerun-if-changed=../../proto");

    Ok(())
}
