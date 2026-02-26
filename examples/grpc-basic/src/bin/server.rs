pub mod hello {
    tonic::include_proto!("hello");
}

use hello::messenger_server::{Messenger, MessengerServer};
use hello::{Ack, Message};
use tonic::{Request, Response, Status};

#[derive(Default)]
struct MessengerService;

#[tonic::async_trait]
impl Messenger for MessengerService {
    async fn send_message(&self, _request: Request<Message>) -> Result<Response<Ack>, Status> {
        println!("Sent ok back");
        Ok(Response::new(Ack { ok: true }))
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:50051".parse()?;
    let service = MessengerService::default();

    tonic::transport::Server::builder()
        .add_service(MessengerServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}
