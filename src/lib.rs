#![feature(generic_associated_types)]
#![feature(type_alias_impl_trait)]
#![feature(io_error_more)]
#![feature(pin_macro)]

pub mod fs;
pub mod io;
pub mod net;
pub mod runtime;

#[cfg(test)]
mod tests {
    use super::{
        fs::{File, OpenOptions},
        io::{Read, ReadAt, ReadAtExt, ReadExt, Write, WriteAt},
        runtime::Runtime,
    };

    #[test]
    fn file() {
        let rt = Runtime::new().unwrap();
        rt.run(async {
            let fname = "/tmp/photonio.txt";

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
        });
    }
}
