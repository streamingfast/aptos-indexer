fn main() {
    println!("cargo:rerun-if-changed=proto");
    tonic_build::configure()
        .out_dir("src/proto")
        .compile(&["proto/substreams.proto"], &["proto"])
        .expect("Failed to compile Substreams proto(s)");

    tonic_build::configure()
        .out_dir("src/proto")
        .compile(&["proto/stats.proto"], &["proto"])
        .expect("Failed to compile Substreams Stats proto(s)");
}
