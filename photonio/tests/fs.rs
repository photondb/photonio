use photonio::{
    fs::{File, OpenOptions},
    io::{Read, ReadAt, Result, Write, WriteAt},
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
}

#[cfg(target_os = "linux")]
#[cfg(feature = "uring")]
#[photonio::test(env_logger = true)]
async fn file2() -> Result<()> {
    use std::os::unix::prelude::OpenOptionsExt;

    use photonio::{
        io::{ReadAtFixed, WriteAtFixed},
        runtime::alloc_uring_buf,
    };

    let path = "/tmp/test2.txt";
    let flags = 0x4000;

    let content = "hello 123";
    let content2 = "1211212";
    let data = content.as_bytes();
    let data2 = content2.as_bytes();
    {
        let mut wfile = photonio::fs::OpenOptions::new()
            .write(true)
            .custom_flags(flags)
            .create(true)
            .truncate(true)
            .open(path)
            .await
            .expect("open file_id: {file_id}'s file fail");

        let mut pos = 0;

        let mut wbuf = alloc_uring_buf(data.len(), 512).unwrap();
        wbuf.as_bytes_mut()[0..data.len()].copy_from_slice(data);
        let buf_idx = wbuf.uring_buffer_index();
        let written = wfile.write_at_fixed(wbuf.as_bytes(), pos, buf_idx).await?;

        pos += written as u64;
        let mut wbuf = alloc_uring_buf(data2.len(), 512).unwrap();
        wbuf.as_bytes_mut()[0..data2.len()].copy_from_slice(data2);
        let buf_idx = wbuf.uring_buffer_index();
        wfile.write_at_fixed(wbuf.as_bytes(), pos, buf_idx).await?;
    }

    {
        let rfile = photonio::fs::OpenOptions::new()
            .read(true)
            .custom_flags(flags)
            .open(path)
            .await
            .expect("open file_id: {file_id}'s file fail");

        let mut rbuf = alloc_uring_buf(data.len(), 512).unwrap();
        let rbuf_idx = rbuf.uring_buffer_index();
        let rslice = rbuf.as_bytes_mut();
        rfile.read_at_fixed(rslice, 0, rbuf_idx).await?;
        let s = String::from_utf8_lossy(&rslice[..data.len()]);
        assert_eq!(s.as_ref(), content);

        let mut rbuf = alloc_uring_buf(data2.len(), 512).unwrap();
        let rbuf_idx = rbuf.uring_buffer_index();
        let rslice = rbuf.as_bytes_mut();
        rfile.read_at_fixed(rslice, 512, rbuf_idx).await?;
        let s = String::from_utf8_lossy(&rslice[..data2.len()]);
        assert_eq!(s.as_ref(), content2);
    }
    Ok(())
}
