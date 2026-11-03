use std::error::Error;
use std::time::Duration;

use tokio::time::timeout;
use tokio_postgres::NoTls;

#[derive(Debug)]
struct InstanceStatus {
    name: String,
    port: u16,
    is_primary: bool,
    lsn: i64,
}

async fn check(name: &str, port: u16) -> Result<InstanceStatus, Box<dyn Error>> {
    let conn_str =
        format!("host=localhost port={port} user=postgres password=postgres dbname=testdb");

    let (client, connection) = timeout(Duration::from_secs(1), tokio_postgres::connect(&conn_str, NoTls))
        .await
        .map_err(|_| "connect timed out")??;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection err {}", e);
        }
    });

    let row = client
        .query_one(
            "SELECT pg_is_in_recovery(),
                    (CASE WHEN pg_is_in_recovery() THEN pg_last_wal_replay_lsn()
                          ELSE pg_current_wal_lsn() END - '0/0'::pg_lsn)::bigint",
            &[],
        )
        .await?;

    let in_recovery: bool = row.get(0);
    let lsn: i64 = row.get(1);

    Ok(InstanceStatus {
        name: name.to_string(),
        port,
        is_primary: !in_recovery,
        lsn,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let status = check("primary", 5442).await?;
    println!("primary \n{status:?}");


    let status = check("replica1", 5443).await?;
    println!("replica1 \n{status:?}");

    let status = check("replica2", 5444).await?;
    println!("replica2 \n{status:?}");

    Ok(())
}
