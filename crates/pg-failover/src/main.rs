use tokio_postgres::{NoTls, Error};

struct InstanceStatus {
    name: String,
    port: u16,
    is_primary: bool,
    lsn: i64,
}

// async fn check(name: String, port: u16) -> Result<InstanceStatus, Box<dyn Error>> {
//     let (client, connection) = tokio_postgres::connect("host=localhost port={port} user=posgres password=postgres dbname=testdb", NoTls);
// }

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_postgres::connect("host=localhost port=5442 user=postgres password=postgres dbname=testdb", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection err {}", e);
        }
    });

    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    println!("{}", value);

    Ok(())
}
