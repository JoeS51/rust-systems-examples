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
            &System::name().unwrap(),
            &System::kernel_version().unwrap(),
            &System::os_version().unwrap(),
            &System::host_name().unwrap(),
        ]);

    let mut processes: Vec<_> = sys.processes().iter().collect();
    processes.sort_by_key(|(_, process)| process.memory());
    processes.reverse();

    for (pid, process) in processes.iter().take(10) {
        let mem_usage = process.memory() as f64 / 1024.0 / 1024.0;
        println!(
            "{:?} {:?} {:?}mb {:?} {:?}",
            pid,
            process.name(),
            mem_usage,
            process.cpu_usage(),
            process.status()
        );
    }

    println!("{table}");
}
