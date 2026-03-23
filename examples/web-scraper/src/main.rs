// Assignment 1: Concurrent Web Fetcher
//
// Objective: Build a CLI tool that fetches multiple URLs concurrently and
// reports results.
//
// Requirements:
//
//  1. Accept a hardcoded list of at least 5 URLs (or take them from
//     command-line args - your choice)
//  2. Fetch all URLs concurrently using tokio::spawn and reqwest
//  3. For each URL, print:
//      - The URL
//      - The HTTP status code (or the error if the request failed)
//      - How long that individual request took
//  4. After all requests complete, print the total elapsed time
//  5. Handle errors gracefully — a single failed URL should not crash the
//     program
//
// Hints:
//
//  - You'll need to add tokio (with full features) and reqwest to your
//    Cargo.toml
//  - std::time::Instant is fine for timing
//  - Think about what type JoinHandle returns and how to collect results
//
// Grading criteria:
//
//  - All URLs fetched concurrently (not sequentially!)
//  - Errors are handled, not unwrap()'d
//  - Clean, idiomatic code

use std::io;
use std::time::Instant;
use tokio::task;
use reqwest;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    println!("enter some urls (q when done):");
    
    let mut urls: Vec<String> = Vec::new();
    
    loop {
        let mut url = String::new();

        io::stdin()
            .read_line(&mut url)
            .expect("Couldn't read line");

        if url.trim() == "q" {
            break;
        }
        urls.push(url);
        println!("------");
    }

    println!("");

    let start_time = Instant::now();
    let mut handles = Vec::new();
    
    for (i, url) in urls.into_iter().enumerate() {
        let handle = tokio::spawn(async move {
            let req = reqwest::get(&url).await.unwrap();

            println!("Result {}:", i);
            println!("URL: {:?}", url);
            println!("Status code: {:?}", req.status());
            println!("");
        });
        handles.push(handle);
    }

    join_all(handles).await;
    
    let duration = start_time.elapsed();
    println!("Duration: {:?}", duration);
}
