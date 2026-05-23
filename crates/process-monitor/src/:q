use clap::Parser;
use comfy_table::Table;
use sysinfo::{Components, Disks, Networks, System};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    #[arg(short, long, default_value = "memory")]
    sort: String,
}

fn main() {
    let args = Args::parse();
    let mut table = Table::new();

    let mut sys = System::new_all();
    sys.refresh_all();

    table
        .set_header(vec![
            "System name",
            "Kernel version",
            "OS version",
            "host name",
        ])
        .add_row(vec![
            System::name().unwrap_or_else(|| "unknown".to_string()),
            System::kernel_version().unwrap_or_else(|| "unknown".to_string()),
            System::os_version().unwrap_or_else(|| "unknown".to_string()),
            System::host_name().unwrap_or_else(|| "unknown".to_string()),
        ]);
    println!("{table}");

    let mut processes: Vec<_> = sys.processes().iter().collect();
    let mut process_table = Table::new();
    process_table.set_header(vec![
        "PID",
        "Name",
        "Memory MiB",
        "CPU %",
        "Run time",
        "Status",
    ]);
    match args.sort.as_str() {
        "memory" => {
            processes.sort_by_key(|(_, process)| process.memory());
            processes.reverse();
        }
        "cpu" => {
            processes.sort_by(|(_, a), (_, b)| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
        }
        _ => {
            eprintln!("unknown sort option: {}", args.sort);
            std::process::exit(1);
        }
    }

    for (pid, process) in processes.iter().take(args.limit) {
        let mem_usage = process.memory() as f64 / 1024.0 / 1024.0;
        let run_time = process.run_time() as f64 / 60.0 / 60.0;
        process_table.add_row(vec![
            pid.to_string(),
            process.name().to_string_lossy().to_string(),
            format!("{:.2} mib", mem_usage),
            process.cpu_usage().to_string(),
            format!("{:.2} hours", run_time),
            process.status().to_string(),
        ]);
    }
    println!("top 10 processes by {}", args.sort);
    println!("{process_table}");
}
