use kimojio::{
    Errno,
    configuration::Configuration,
    operations::{self, OFlags},
};

#[kimojio::main]
async fn main() -> Result<(), Errno> {
    println!("copying file");

    let read_flags = OFlags::RDONLY;
    let write_flags = OFlags::CREATE | OFlags::RDWR;
    let rd_fd = operations::open(c"input.txt", read_flags, 0o644.into()).await?;
    let w_fd = operations::open(c"output.txt", write_flags, 0o644.into()).await?;

    let mut buffer = [0u8; 32];
    let read_res = operations::read(&rd_fd, &mut buffer).await?;

    println!("read res {}", read_res);

    operations::write(&w_fd, &buffer).await?;

    operations::close(rd_fd).await?;
    Ok(())
}
