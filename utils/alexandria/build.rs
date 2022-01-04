use protoc_rust::Customize;
use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/io/proto_gen", out_dir);

    if Path::new(&out_path).exists() {
        let _ = fs::remove_dir_all(&out_path);
    }
    fs::create_dir_all(&out_path).unwrap();

    protoc_rust::Codegen::new()
        .customize(Customize {
            gen_mod_rs: Some(true),
            ..Default::default()
        })
        .out_dir(out_path)
        .input("schema/chunk.proto")
        .input("schema/encrypted.proto")
        .input("schema/table.proto")
        .run()
        .expect("Failed to compile protobuf schemas!");
}
