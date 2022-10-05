use photonio::{
    fs::File,
    io::{Read, ReadAt, Result, Write, WriteAt},
};

#[photonio::test(env_logger = true)]
async fn file() -> Result<()> {
    let path = "/tmp/test.txt";

    let mut file = File::create(path).await?;
    file.write(b"hello").await?;
    file.write_at(b"world", 5).await?;

    let mut buf = [0; 10];
    let mut file = File::open(path).await?;
    file.read(&mut buf[..5]).await?;
    file.read_at(&mut buf[5..], 5).await?;
    assert_eq!(&buf, b"helloworld");

    Ok(())
}
