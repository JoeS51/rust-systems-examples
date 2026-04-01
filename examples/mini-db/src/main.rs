use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(PartialEq)]
enum OperationType {
    Set,
    Get,
}

struct Operation {
    operation_type: OperationType,
    key: String,
    value: Option<String>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut n = 0;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("db{}.log", n))?;

    let mut writer = BufWriter::new(file);
    let mut num_records = 0;
    loop {
        println!("Enter DB operation (e.g. db set 1 hi) ");
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let curr_op = parse_line(&input)?;

        match curr_op.operation_type {
            OperationType::Get => {
                let scan_res = search_file(&curr_op.key, n)?;
                println!("{}", scan_res);
            }
            OperationType::Set => {
                let record = format!("{} {}\n", curr_op.key, curr_op.value.unwrap());
                writer.write_all(record.as_bytes())?;

                writer.flush()?;

                num_records += 1;
            }
        }

        // Compaction
        if num_records > 3 {
            compact_old_log(format!("db{}.log", n))?;
            n += 1;
            file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("db{}.log", n))?;
            println!("{:?}", file);

            writer = BufWriter::new(file);
            println!("{:?}", writer);
            num_records = 0;
        }
    }
}

fn parse_line(input: &str) -> Result<Operation, Box<dyn Error>> {
    let vec: Vec<&str> = input.split_whitespace().collect();

    match vec.as_slice() {
        ["db", "get", key] => Ok(Operation {
            operation_type: OperationType::Get,
            key: key.to_string(),
            value: None,
        }),
        ["db", "set", key, value] => Ok(Operation {
            operation_type: OperationType::Set,
            key: key.to_string(),
            value: Some(value.to_string()),
        }),
        _ => Err("invalid input".into()),
    }
}

fn search_file(key: &str, n: i32) -> Result<String, Box<dyn Error>> {
    for segment in (0..n).rev() {
        let file = File::open(format!("db{}.log", segment))?;
        let reader = BufReader::new(file);

        let mut found = None;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split_whitespace().collect();

            if parts.len() == 2 && parts[0] == key {
                found = Some(parts[1].to_string());
            }
        }

        if let Some(value) = found {
            return Ok(value);
        }
    }

    Ok("not found".to_string())
}

fn compact_old_log(file_name: String) -> Result<String, Box<dyn Error>> {
    let file = File::open(file_name)?;
    let reader = BufReader::new(&file);
    let mut entries: HashMap<String, String> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        entries.insert(parts[0].to_string(), parts[1].to_string());
    }

    let compacted_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("compacted_log.log")?;
    let mut writer = BufWriter::new(compacted_file);

    for entry in entries {
        let record = format!("{} {}\n", entry.0, entry.1);
        writer.write_all(record.as_bytes())?;
        writer.flush()?;
    }

    Ok("test".to_string())
}
