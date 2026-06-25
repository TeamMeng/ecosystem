fn main() {
    tonic_build::configure()
        .out_dir("src")
        .compile_protos(&["proto/hello.proto"], &["proto"])
        .unwrap();
}
