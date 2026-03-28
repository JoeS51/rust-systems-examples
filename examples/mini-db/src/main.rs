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
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("db.log")?;

    let mut writer = BufWriter::new(&file);

    loop {
        println!("Enter DB operation (e.g. db set 1 hi) ");
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let curr_op = parse_line(&input)?;

        match curr_op.operation_type {
            OperationType::Get => {
                let scan_res = search_file(&curr_op.key)?;
                println!("{}", scan_res);
            }
            OperationType::Set => {
                let record = format!("{} {}\n", curr_op.key, curr_op.value.unwrap());
                writer.write_all(record.as_bytes())?;

                writer.flush()?;
            }
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

fn search_file(key: &str) -> Result<String, Box<dyn Error>> {
    let file = File::open("db.log")?;
    let reader = BufReader::new(file);

    let mut res = String::from("-1");
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts[0] == key {
            res = parts[1].to_string();
        }
    }

    Ok(res)
}
