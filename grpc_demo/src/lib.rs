// Generated protobuf messages + tonic client/server stubs for greeter.proto.
// (`cargo build` runs build.rs, which invokes protoc via tonic-prost-build.)
pub mod greeter {
    tonic::include_proto!("greeter");
}

// The tap's decoding layers live in the library so they could be unit-tested;
// the binaries are thin wrappers.
pub mod hexdump;
pub mod hpack;
pub mod hpack_huffman;
pub mod protowire;
