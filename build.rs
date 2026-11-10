use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let base_proto = flare_proto_include_dir(&manifest);
    let grpc_proto = manifest.join("proto");

    let protos = vec![
        grpc_proto.join("access_gateway.proto"),
        grpc_proto.join("auth_revoke.proto"),
        grpc_proto.join("control_plane.proto"),
        grpc_proto.join("conversation_service.proto"),
        grpc_proto.join("capability_service.proto"),
        grpc_proto.join("media_service.proto"),
        grpc_proto.join("message_service.proto"),
        grpc_proto.join("online.proto"),
        grpc_proto.join("push_service.proto"),
        grpc_proto.join("router.proto"),
        grpc_proto.join("storage_service.proto"),
        grpc_proto.join("sync_service.proto"),
    ];

    for p in &protos {
        println!("cargo:rerun-if-changed={}", p.display());
    }
    println!("cargo:rerun-if-changed={}", base_proto.display());
    println!("cargo:rerun-if-env-changed=PROTOC_INCLUDE");

    // 基础 proto 目录优先，避免与 `*_service.proto` 中的 `import "message.proto"` 等同名引用歧义
    let includes = proto_include_dirs(&base_proto, &grpc_proto);

    let mut config = tonic_prost_build::configure();
    config = config
        .build_client(true)
        .build_server(true)
        .compile_well_known_types(false)
        .extern_path(".flare.common.v1", "::flare_proto::flare::common::v1")
        // 为 HTTP 网关直连 proto message 提供 serde 序列化能力
        .type_attribute(
            ".flare.media.v1.GenerateUploadUrlRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.GenerateUploadUrlResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.GetFileUrlRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.GetFileInfoRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.DeleteFileRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.DeleteFileResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.CreateReferenceRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.DeleteReferenceRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.CleanupOrphanedAssetsRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.CleanupOrphanedAssetsResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.SetObjectAclRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.AccessControlEntry",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.DescribeBucketRequest",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            ".flare.media.v1.DescribeBucketResponse",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        );

    // 控制面 outbox 以 JSON 存投影载荷,整个包派生 serde
    config = config.type_attribute(
        ".flare.control.v1",
        "#[derive(serde::Serialize, serde::Deserialize)]",
    );

    let proto_paths: Vec<_> = protos
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();

    config.compile_protos(&proto_paths, &includes)?;
    Ok(())
}

fn flare_proto_include_dir(manifest: &Path) -> PathBuf {
    std::env::var_os("DEP_FLARE_PROTO_PROTO_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest.join("..").join("flare-protocol").join("proto"))
}

fn proto_include_dirs(base_proto: &Path, grpc_proto: &Path) -> Vec<String> {
    let mut includes = vec![
        base_proto.to_string_lossy().to_string(),
        grpc_proto.to_string_lossy().to_string(),
    ];

    for dir in well_known_proto_include_dirs() {
        let dir = dir.to_string_lossy().to_string();
        if !includes.iter().any(|existing| existing == &dir) {
            includes.push(dir);
        }
    }

    includes
}

fn well_known_proto_include_dirs() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(include_paths) = std::env::var_os("PROTOC_INCLUDE") {
        candidates.extend(std::env::split_paths(&include_paths));
    }

    candidates.extend([
        PathBuf::from("/usr/include"),
        PathBuf::from("/usr/local/include"),
        PathBuf::from("/opt/homebrew/include"),
    ]);

    candidates
        .into_iter()
        .filter(|dir| dir.join("google/protobuf/timestamp.proto").is_file())
        .collect()
}
