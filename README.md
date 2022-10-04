# PhotonIO

![crates](https://img.shields.io/crates/v/photonio?style=for-the-badge)
![docs](https://img.shields.io/docsrs/photonio?style=for-the-badge)

PhotonIO is a runtime for asynchronous applications in Rust.

## Features

- Support asynchronous filesystem and networking operations based on io-uring.

## Examples

```rust
use photonio::{fs::File, io::Write};

#[photonio::main]
async fn main() -> std::io::Result<()> {
    let mut file = File::create("hello.txt").await?;
    file.write(b"hello").await?;
    Ok(())
}
```