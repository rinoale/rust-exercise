//! gRPC server. Listens on 50051 (plaintext HTTP/2, no TLS, so the tap can
//! read it).
//!
//!   cargo run --bin server

use grpc_demo::greeter::greeter_server::{Greeter, GreeterServer};
use grpc_demo::greeter::{HelloReply, HelloRequest};
use std::time::Duration;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{transport::Server, Request, Response, Status};

struct GreeterService;

#[tonic::async_trait]
impl Greeter for GreeterService {
    // Unary handler: return one message.
    async fn say_hello(&self, request: Request<HelloRequest>) -> Result<Response<HelloReply>, Status> {
        let meta: Vec<String> = request
            .metadata()
            .iter()
            .filter_map(|kv| match kv {
                tonic::metadata::KeyAndValueRef::Ascii(k, v) if !k.as_str().starts_with("grpc-") => {
                    Some(format!("{}={:?}", k, v.to_str().unwrap_or("?")))
                }
                _ => None,
            })
            .collect();
        let req = request.into_inner();
        eprintln!("[server] SayHello name={:?} times={}", req.name, req.times);
        eprintln!("[server]   request metadata: {{{}}}", meta.join(", "));

        let mut resp = Response::new(HelloReply {
            message: format!("Hello, {}!", req.name),
            count: 1,
        });
        // Metadata set here is sent as the *initial* HEADERS frame of the response.
        resp.metadata_mut().insert("x-served-by", "rust-greeter".parse().unwrap());
        Ok(resp)
    }

    type CountDownStream = ReceiverStream<Result<HelloReply, Status>>;

    // Server-streaming handler: one DATA frame per message sent to the channel.
    async fn count_down(&self, request: Request<HelloRequest>) -> Result<Response<Self::CountDownStream>, Status> {
        let req = request.into_inner();
        eprintln!("[server] CountDown name={:?} times={}", req.name, req.times);
        let (tx, rx) = tokio::sync::mpsc::channel(4);
        tokio::spawn(async move {
            for i in (1..=req.times).rev() {
                let reply = HelloReply { message: format!("{}: {}", req.name, i), count: i as i64 };
                if tx.send(Ok(reply)).await.is_err() {
                    return; // client went away
                }
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
            let _ = tx
                .send(Ok(HelloReply { message: format!("{}: liftoff", req.name), count: 0 }))
                .await;
        });
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:50051".parse()?;
    eprintln!("[server] listening on {addr} (plaintext h2c)");
    Server::builder()
        .add_service(GreeterServer::new(GreeterService))
        .serve(addr)
        .await?;
    Ok(())
}
