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
    let bytes_read = operations::read(&rd_fd, &mut buffer).await?;

    println!("read res {}", bytes_read);

    operations::write(&w_fd, &buffer[..bytes_read]).await?;

    operations::close(rd_fd).await?;
    operations::close(w_fd).await?;
    Ok(())
}
