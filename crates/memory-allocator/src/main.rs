#[repr(C)]
struct Block {
    size: usize,
    next: *mut Block,
}

fn main() {
    println!("Hello, world!");
}
