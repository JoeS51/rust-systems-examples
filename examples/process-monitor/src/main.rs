use sysinfo::{Components, Disks, Networks, System};

fn main() {
    let mut sys = System::new_all();

    sys.refresh_all();

    println!("total mem: {} bytes", sys.total_memory());
}
