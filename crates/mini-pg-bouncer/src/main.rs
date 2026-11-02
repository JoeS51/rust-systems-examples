use std::sync::*;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
struct Connection {
    id: usize,
}

struct Pool {
    connections: Mutex<Vec<Connection>>,
    available: Condvar,
}

impl Pool {
    fn new(size: usize) -> Pool {
        let mut v = Vec::new();
        for i in 0..size {
            v.push(
                Connection {
                    id: i,
                }
            );
        }
        Pool {
            connections: Mutex::new(v),
            available: Condvar::new(),
        }
    }

    fn acquire(&self) -> Connection {
        let mut guard = self.connections.lock().unwrap(); 
        while guard.is_empty() {
            guard = self.available.wait(guard).unwrap();
        }
        guard.pop().unwrap()
    }

    fn release(&self, conn: Connection) {
        let mut guard = self.connections.lock().unwrap();
        guard.push(conn);
        self.available.notify_one();
    }
}

fn main() {
    let pool = Arc::new(Pool::new(3));
    let mut handles = Vec::new();

    for c in 0..10 {
        let pool = Arc::clone(&pool);
        let handle = thread::spawn(move || {
            let conn = pool.acquire(); 
            println!("got connection {}", conn.id);
            thread::sleep(Duration::from_millis(100));
            println!("released connection {}", conn.id);
            pool.release(conn);
        });
        handles.push(handle);
    }

    for h in handles {
        h.join().unwrap();
    }
}
