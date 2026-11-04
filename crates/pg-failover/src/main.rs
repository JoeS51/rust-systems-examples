use std::error::Error;
use std::time::Duration;

use tokio::time::timeout;
use tokio_postgres::NoTls;

const NODES: [(&str, u16); 3] = [("primary", 5442), ("replica1", 5443), ("replica2", 5444)];

#[derive(Debug)]
struct InstanceStatus {
    name: String,
    port: u16,
    is_primary: bool,
    lsn: i64,
}

async fn check(name: &str, port: u16) -> Result<InstanceStatus, Box<dyn Error>> {
    let conn_str = format!("host=localhost port={port} user=postgres password=postgres dbname=testdb");

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

async fn failover(instances: &[InstanceStatus]) -> Result<(), Box<dyn Error>> {
    // find the highest lsn replica and promote it
    println!("performing failover!!!!");
    let replica = instances.iter().max_by_key(|r| r.lsn).ok_or("no reachable replicas")?;
    println!("promoting {} (port {}, lsn {})", replica.name, replica.port, replica.lsn);

    let conn_str = format!("host=localhost port={} user=postgres password=postgres dbname=testdb", replica.port);

    let (client, connection) = timeout(Duration::from_secs(1), tokio_postgres::connect(&conn_str, NoTls))
        .await
        .map_err(|_| "connect timed out")??;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection err {}", e);
        }
    });

    let promote = client.query_one("SELECT pg_promote()",&[]).await?;
    let promote_result: bool = promote.get(0);
    println!("promotion succeeded: {}", promote_result);

    // TODO: STONITH the old primary if this succeeded

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let mut instance_statuses = Vec::new();

        for node in NODES {
            let status = check(node.0, node.1).await;
            match status {
                Ok(stat) =>  {
                    println!("{} \n{stat:?}", node.0);
                    instance_statuses.push(stat);
                },
                Err(_) => println!("{} is down", node.0),
            }
        }

        let has_primary = instance_statuses.iter().any(|s| s.is_primary);

        if !has_primary {
            match failover(&instance_statuses).await {
                Ok(_) => println!("failover complete"),
                Err(e) => println!("failover failed {}", e),
            }
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
