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

    for (pid, process) in sys.processes() {
        println!("[{pid}] {:?} {:?}", process.name(), process.disk_usage());
    }

    println!("{table}");
}
