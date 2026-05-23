pub mod hello {
    tonic::include_proto!("hello");
}

use hello::Message;
use hello::messenger_client::MessengerClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = MessengerClient::connect("http://127.0.0.1:50051").await?;

    let req = Message {
        title: "hello".to_string(),
        id: 1,
        message: "from client".to_string(),
    };

    let response = client.send_message(req).await?.into_inner();
    println!("ok: {}", response.ok);

    Ok(())
}
