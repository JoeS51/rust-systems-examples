use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

fn main() {
    let mut map: HashMap<String, u32> = HashMap::new();
    map.insert("Joe".to_string(), 100);
    map.insert("Alice".to_string(), 0);
    map.insert("Bob".to_string(), 50);
    let bank = Arc::new(Mutex::new(map));
    let mut handles = vec![];
    for i in 0..10 {
        for key in map.keys() {}
        let tx1 = Arc::clone(&bank);
        let handle = thread::spawn(move || {});
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("{:?}", bank);
}
