use photonio::{
    fs::{File, OpenOptions},
    io::{Read, ReadAt, Write, WriteAt},
};

#[photonio::test(env_logger = true)]
async fn file() {
    let path = "/tmp/test.txt";

    let mut file = File::create(path).await.unwrap();
    file.write(b"hello").await.unwrap();
    file.write_at(b"world", 5).await.unwrap();

    let mut buf = [0; 10];
    let mut file = File::open(path).await.unwrap();
    file.read(&mut buf[..5]).await.unwrap();
    file.read_at(&mut buf[5..], 5).await.unwrap();
    assert_eq!(&buf, b"helloworld");

    let file = OpenOptions::new().write(true).open(path).await.unwrap();
    let meta = file.metadata().await.unwrap();
    assert_eq!(meta.len(), 10);
    file.set_len(5).await.unwrap();
    let meta = file.metadata().await.unwrap();
    assert_eq!(meta.len(), 5);

    use std::os::unix::prelude::MetadataExt;
    let dev = meta.dev();
    assert_ne!(dev, 0);
}
