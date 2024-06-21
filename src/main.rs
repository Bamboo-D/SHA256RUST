use crossbeam::channel::{unbounded, Receiver};
use sha2::{Digest, Sha256};
use std::env;
use std::fmt::Write;
use std::sync::{Arc, Mutex};
use std::thread;

const PREFIX: &str = "hello/";
const SUFFIX: &str = "+worlds";
const THREAD_COUNT: usize = 14;

fn calculate_sha256(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    let mut hex_string = String::new();
    for byte in result {
        write!(&mut hex_string, "{:02x}", byte).expect("Unable to write");
    }
    hex_string
}

fn worker(id: usize, rx: Receiver<usize>, min_hash: Arc<Mutex<(String, String)>>) {
    for i in rx {
        let candidate = format!("{}{}{}", PREFIX, i, SUFFIX);

        if i % 10000000 == 0 {
            println!("[ now: {} ]", i);
        }

        let hash = calculate_sha256(&candidate);

        let mut min_hash_guard = min_hash.lock().unwrap();
        if hash < min_hash_guard.0 {
            min_hash_guard.0 = hash.clone();
            min_hash_guard.1 = candidate.clone();
            println!("({}) hash: {} for string: {}", id, min_hash_guard.0, i);
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <start> <end>", args[0]);
        return;
    }

    let start: usize = args[1].parse().expect("Invalid start value");
    let end: usize = args[2].parse().expect("Invalid end value");

    let (tx, rx) = unbounded();
    let min_hash = Arc::new(Mutex::new((String::from("f".repeat(64)), String::new())));

    let mut handles = vec![];

    for id in 0..THREAD_COUNT {
        let rx = rx.clone();
        let min_hash = Arc::clone(&min_hash);
        let handle = thread::spawn(move || {
            worker(id, rx, min_hash);
        });
        handles.push(handle);
    }

    for i in start..=end {
        tx.send(i).expect("Unable to send on channel");
    }

    drop(tx);

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    let final_min_hash = min_hash.lock().unwrap();
    println!(
        "[Final]\n min hash: {}\n for string: {}",
        final_min_hash.0, final_min_hash.1
    );
}
