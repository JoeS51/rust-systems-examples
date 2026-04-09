use std::collections::HashMap;
use std::error::Error;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};

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

struct Segment {
    id: u32,
    index: HashMap<String, usize>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Clear previously written db files
    clear_db_files()?;

    let mut n: u32 = 0;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(format!("db{}.log", n))?;

    let mut writer = BufWriter::new(file);
    let mut num_records = 0;

    let index: HashMap<String, usize> = HashMap::new();
    let mut segments: Vec<Segment> = Vec::new();
    segments.push(Segment {
        id: n,
        index: HashMap::new(),
    });
    let mut cursor: usize = 0;

    loop {
        println!("Enter DB operation (e.g. db set 1 hi) ");
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read input");

        let mut curr_op = parse_line(&input)?;

        match curr_op.operation_type {
            OperationType::Get => {
                let scan_res = search_file(&curr_op.key, &segments, n)?;
                println!("{scan_res}");
            }
            OperationType::Set => {
                let record = format!("{} {}\n", curr_op.key, curr_op.value.clone().unwrap());
                writer.write_all(record.as_bytes())?;

                writer.flush()?;

                num_records += 1;

                segments
                    .get_mut(n as usize)
                    .unwrap()
                    .index
                    .insert(curr_op.key.clone(), cursor);

                cursor += curr_op.key.len() + curr_op.value.clone().unwrap().len() + 2;
            }
        }

        // Compaction
        if num_records > 3 {
            n += 1;
            file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(format!("db{}.log", n))?;
            println!("{:?}", file);

            writer = BufWriter::new(file);
            compact_old_log(n - 1)?;

            segments.push(Segment {
                id: n,
                index: HashMap::new(),
            });

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

fn search_file(key: &str, segments: &Vec<Segment>, n: u32) -> Result<String, Box<dyn Error>> {
    if let Some(&offset) = segments.get(n as usize).unwrap().index.get(key) {
        println!("offset: {:?}", offset);
        let mut file = File::open(format!("db{}.log", n))?;
        file.seek(SeekFrom::Start(offset as u64))?;

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let parts: Vec<&str> = line.split_whitespace().collect();
        let id = parts[0].to_string();
        let value = parts[1].to_string();
        if parts.len() == 2 && id == key {
            return Ok(value.to_string());
        } else {
            return Ok("corrupted data or somethign".to_string());
        }
    } else {
        Ok("not found".to_string())
    }
}

fn compact_old_log(segment: u32) -> Result<String, Box<dyn Error>> {
    let original = format!("db{}.log", segment);
    let temp = format!("db{}.tmp", segment);

    let file = File::open(&original)?;
    let reader = BufReader::new(&file);
    let mut entries: HashMap<String, String> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();
        entries.insert(parts[0].to_string(), parts[1].to_string());
    }

    {
        let temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temp)?;

        let mut writer = BufWriter::new(temp_file);

        for (key, value) in entries {
            let record = format!("{} {}\n", key, value);
            writer.write_all(record.as_bytes())?;
        }

        writer.flush()?;
    }

    fs::rename(&temp, &original)?;

    Ok("test".to_string())
}

fn clear_db_files() -> Result<(), Box<dyn std::error::Error>> {
    let entries = fs::read_dir(".")?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("db") && name.ends_with(".log") {
                fs::remove_file(&path)?;
            }
        }
    }

    Ok(())
}
