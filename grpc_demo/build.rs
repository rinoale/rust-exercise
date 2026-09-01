fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generates the message structs (prost) and client/server stubs (tonic)
    // into OUT_DIR; src/lib.rs pulls them in with include_proto!.
    tonic_prost_build::compile_protos("proto/greeter.proto")?;
    Ok(())
}
