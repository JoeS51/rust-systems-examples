// Same pool as main.rs, but each item is a real Postgres connection.
//
// Run with:
//   DATABASE_URL="host=localhost user=postgres password=YOURPASS" cargo run --bin pg_pool

use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use postgres::{Client, NoTls};

struct Pool {
    connections: Mutex<Vec<Client>>,
    available: Condvar,
}

impl Pool {
    // Opens `size` real connections up front. Any failure (DB down,
    // bad password, ...) is returned to the caller via `?`.
    fn new(url: &str, size: usize) -> Result<Pool, postgres::Error> {
        let mut v = Vec::with_capacity(size);
        for _ in 0..size {
            v.push(Client::connect(url, NoTls)?);
        }
        Ok(Pool {
            connections: Mutex::new(v),
            available: Condvar::new(),
        })
    }

    // Identical logic to your version: lock, wait while empty, pop.
    fn acquire(&self) -> PooledClient<'_> {
        let mut guard = self.connections.lock().unwrap();
        while guard.is_empty() {
            guard = self.available.wait(guard).unwrap();
        }
        let client = guard.pop().expect("loop guarantees non-empty");
        PooledClient {
            client: Some(client),
            pool: self,
        }
    }

    // Private-ish: only the guard's Drop calls this.
    fn release(&self, client: Client) {
        // The temporary MutexGuard is dropped at the end of this line,
        // so we notify *after* unlocking.
        self.connections.lock().unwrap().push(client);
        self.available.notify_one();
    }
}

// The guard: hands out a connection, returns it automatically on drop.
struct PooledClient<'a> {
    client: Option<Client>, // Option so Drop can .take() ownership
    pool: &'a Pool,
}

// Lets callers use the guard like a `&Client`.
impl Deref for PooledClient<'_> {
    type Target = Client;
    fn deref(&self) -> &Client {
        self.client.as_ref().unwrap()
    }
}

// Lets callers use the guard like a `&mut Client` (needed for .query()).
impl DerefMut for PooledClient<'_> {
    fn deref_mut(&mut self) -> &mut Client {
        self.client.as_mut().unwrap()
    }
}

impl Drop for PooledClient<'_> {
    fn drop(&mut self) {
        if let Some(client) = self.client.take() {
            self.pool.release(client);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "host=localhost user=postgres".to_string());

    let pool = Arc::new(Pool::new(&url, 3)?);
    println!("pool ready with 3 connections");

    let mut handles = Vec::new();
    for t in 0..10 {
        let pool = Arc::clone(&pool);
        handles.push(thread::spawn(move || {
            let mut conn = pool.acquire();

            // pg_backend_pid() = the id of the Postgres server process
            // handling this connection. With 3 connections you should
            // only ever see 3 distinct pids.
            let row = conn.query_one("SELECT pg_backend_pid()", &[]).unwrap();
            let pid: i32 = row.get(0);
            println!("thread {t} running on backend pid {pid}");

            // Simulate a slow query so threads have to wait for each other.
            conn.execute("SELECT pg_sleep(0.1)", &[]).unwrap();

            // conn dropped here -> returned to the pool automatically
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    Ok(())
}
