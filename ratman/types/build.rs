
// We gate this builder on the inclusion of the "proto" feature, which
// is disabled on docs.rs.  In that case we include an empty main

#[cfg(feature = "proto")]
fn main() {
    use std::{env, fs, path::Path};
    use protoc_rust::Customize;
    
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/proto_gen", out_dir);

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
        .input("proto/message.proto")
        .input("proto/api.proto")
        .run()
        .expect("Failed to compile protobuf schemas!");
}

#[cfg(not(feature = "proto"))]
fn main() {}
