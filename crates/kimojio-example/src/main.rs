use kimojio::{
    Errno,
    configuration::Configuration,
    operations::{self, OFlags},
};

#[kimojio::main]
async fn main() -> Result<(), Errno> {
    println!("start");
    let flags = OFlags::CREATE | OFlags::RDWR;
    let fd = operations::open(c"/tmp/example.txt", flags, 0o644.into()).await?;
    let written = operations::write(&fd, b"hello").await?;
    assert_eq!(written, 5);
    operations::close(fd).await?;
    Ok(())
}
