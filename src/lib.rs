pub mod api;
// `hello.rs` is generated gRPC code. Newer clippy flags what the generator
// emits: `tonic::Status` is large as an error variant in the client, and
// `#[async_trait]` adds a second `#[must_use]` to methods that already have
// one on the server side.
#[allow(clippy::result_large_err, clippy::double_must_use)]
pub mod hello;
pub mod model;
pub mod state;
pub mod worker;
