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
        io::{Read, Write},
        runtime::Runtime,
    };

    #[test]
    fn file() {
        let rt = Runtime::new().unwrap();
        rt.run(async {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .open("photonio.txt")
                .await
                .unwrap();
            file.write(b"hello").await.unwrap();
        });
    }
}
