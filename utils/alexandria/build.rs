fn main() {
    capnpc::CompilerCommand::new()
        .file("proto/encrypted.capnp")
        .file("proto/chunk.capnp")
        .default_parent_module(vec!["io".into()])
        .run()
        .expect("compiling schema");
}
