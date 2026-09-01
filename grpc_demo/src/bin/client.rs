//! gRPC client. By default it talks to the tap on 50052, which forwards to
//! the real server on 50051 while printing every byte in both directions.
//!
//!   cargo run --bin client                        # through the tap
//!   cargo run --bin client -- localhost:50051     # straight to the server

use grpc_demo::greeter::greeter_client::GreeterClient;
use grpc_demo::greeter::{HelloReply, HelloRequest};
use prost::Message;
use tonic::codegen::http::uri::PathAndQuery;
use tonic::transport::Endpoint;
use tonic::Request;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = std::env::args().nth(1).unwrap_or_else(|| "localhost:50052".into());
    let channel = Endpoint::from_shared(format!("http://{target}"))?.connect().await?;
    let mut client = GreeterClient::new(channel.clone());

    // --- What the client will actually put on the wire ---------------------
    let req = HelloRequest { name: "kevin".into(), times: 3 };
    let bytes = req.encode_to_vec();
    println!("[client] request message   : {req:?}");
    println!("[client] protobuf bytes    : {}  ({} bytes)", hex(&bytes), bytes.len());
    println!("[client] gRPC frame on wire: 00 {:08x} {}", bytes.len(), hex(&bytes));
    println!("[client]                     ^compressed-flag  ^big-endian length  ^message");
    println!();

    // --- Unary call ---------------------------------------------------------
    println!("=== SayHello (unary) ===");
    let mut request = Request::new(req.clone());
    request.metadata_mut().insert("x-demo-header", "hi-from-client".parse()?);
    let reply = client.say_hello(request).await?.into_inner();
    println!("[client] reply            : {reply:?}");
    println!("[client] reply bytes      : {}", hex(&reply.encode_to_vec()));
    println!();

    // --- Server-streaming call ----------------------------------------------
    println!("=== CountDown (server streaming) ===");
    let mut stream = client.count_down(Request::new(req.clone())).await?.into_inner();
    while let Some(r) = stream.message().await? {
        println!("[client] <- {:?} (count={})", r.message, r.count);
    }
    println!();

    // --- An error, so you can see grpc-status != 0 in the trailers -----------
    println!("=== Deliberate error (unknown method) ===");
    let mut grpc = tonic::client::Grpc::new(channel);
    grpc.ready().await?;
    let codec = tonic_prost::ProstCodec::<HelloRequest, HelloReply>::default();
    let path = PathAndQuery::from_static("/greeter.Greeter/NoSuchMethod");
    match grpc.unary(Request::new(req), path, codec).await {
        Ok(_) => println!("[client] unexpectedly succeeded?!"),
        Err(status) => println!(
            "[client] status={:?} (code {}) details={:?}",
            status.code(),
            status.code() as i32,
            status.message()
        ),
    }
    Ok(())
}
