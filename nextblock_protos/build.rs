//use tonic_build::configure;

fn main() {
    let _ = tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["proto/api.proto"], &["proto"]);
}
