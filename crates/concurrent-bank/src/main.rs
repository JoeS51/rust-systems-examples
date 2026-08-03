use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

fn main() {
    let bank = Arc::new(Mutex::new(HashMap::new()));
    let tx1 = Arc::clone(&bank);
    let handle = thread::spawn(move || {
        tx1.lock().unwrap().insert("test", "test");
    });

    handle.join().unwrap();

    println!("{:?}", bank);
}
