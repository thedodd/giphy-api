use prost_build;

fn main() {
    prost_build::Config::new()
        .out_dir("./src/proto")
        .compile_protos(&["../api.proto"], &[".."])
        .unwrap_or_else(|err| panic!("Protobuf compilation failed: {}", err));
}
