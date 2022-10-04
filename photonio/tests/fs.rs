use photonio::{
    fs::{File, OpenOptions},
    io::{Read, ReadAt, Write, WriteAt},
};

#[photonio::test(env_logger = true)]
async fn file() {
    let path = "/tmp/test.txt";

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await
        .unwrap();
    file.write(b"hello").await.unwrap();
    file.write_at(b"world", 5).await.unwrap();

    let mut file = File::open(path).await.unwrap();
    let mut buf = [0; 10];
    file.read(&mut buf).await.unwrap();
    file.read_at(&mut buf[5..], 5).await.unwrap();
    assert_eq!(&buf, b"helloworld");
}
