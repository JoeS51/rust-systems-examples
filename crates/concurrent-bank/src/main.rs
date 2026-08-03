use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let bank = Arc::new(Mutex::new(HashMap::new()));
    let mut handles = vec![];
    for i in 1..4 {
        let tx1 = Arc::clone(&bank);
        let handle = thread::spawn(move || {
            tx1.lock().unwrap().insert(format!("test{}", i), "test");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("{:?}", bank);
}
