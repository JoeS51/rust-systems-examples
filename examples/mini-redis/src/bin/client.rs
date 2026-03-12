use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

#[derive(Debug)]
enum Command {
    Set { key: String, value: Bytes },
    Get { key: String },
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
}
