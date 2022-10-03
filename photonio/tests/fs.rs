use photonio::{
    fs::{File, OpenOptions},
    io::{Read, ReadAt, ReadAtExt, ReadExt, Write, WriteAt},
};

#[photonio::test(env_logger = true)]
async fn file() {
    let fname = "/tmp/test.txt";

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(fname)
        .await
        .unwrap();
    file.write(b"hello").await.unwrap();
    file.write_at(b"world", 5).await.unwrap();

    let mut file = File::open(fname).await.unwrap();
    let mut buf = [0; 10];
    file.read(&mut buf).await.unwrap();
    file.read_at(&mut buf[5..], 5).await.unwrap();
    assert_eq!(&buf, b"helloworld");
    buf.fill(0);
    file.read_exact(&mut buf).await.unwrap();
    file.read_exact_at(&mut buf[5..], 5).await.unwrap();
    assert_eq!(&buf, b"helloworld");
}
