use comfy_table::Table;
use sysinfo::{Components, Disks, Networks, System};

fn main() {
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
    process_table.set_header(vec!["PID", "Name", "Memory MiB", "CPU %", "Status"]);
    processes.sort_by_key(|(_, process)| process.memory());
    processes.reverse();

    for (pid, process) in processes.iter().take(10) {
        let mem_usage = process.memory() as f64 / 1024.0 / 1024.0;
        process_table.add_row(vec![
            pid.to_string(),
            process.name().to_string_lossy().to_string(),
            mem_usage.to_string(),
            process.cpu_usage().to_string(),
            process.status().to_string(),
        ]);
    }
    println!("top 10 processes by memory");
    println!("{process_table}");
}
