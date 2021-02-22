use capnpc::CompilerCommand as Cc;

fn main() {
    Cc::new()
        .file("schema/types.capnp") // base wire wrapper
        .run()
        .expect("Failed compiling schema/carrier.capnp!");
}
