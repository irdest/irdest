# async-eris

Async Rust version of the [ERIS](https://eris.codeberg.page/spec/)
specification `v1.0.0`.  Both block encoding and decoding happens via
asynchronous streams.

To use this functionality you have to implement the `BlockStorage`
async-trait for your storage backend.  `eris::encode(..)` takes an
`AsyncRead + Unpin` type.

**Note** because async-eris is being written for Irdest specifically
we MAY add out-of-specification block sizes to experiment with
different transport slicing mechanisms.  We are also in close contact
with the authors of the specification, so any block size that is
deemed usful MAY becobe part of a future spec version.  Keep this in
mind when potentially using an un-supported block size and
inter-operating with another ERIS implementation!

## Tests

Some tests are included in this library.  You can run them via `cargo
test`, or running one of the examples via `cargo run --example
hello-world`


## Questions?

Want to adapt async-eris in your project but have questions about
something?  Come by our [Matrix channel]() to talk about!

