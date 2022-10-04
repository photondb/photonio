# PhotonIO

[![crates][crates-badge]][crates-url]
[![docs][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/photonio?style=flat-square
[crates-url]: https://crates.io/crates/photonio
[docs-badge]: https://img.shields.io/docsrs/photonio?style=flat-square
[docs-url]: https://docs.rs/photonio/latest/photonio

PhotonIO is an efficient runtime for asynchronous applications in Rust.

## Features

- Asynchronous filesystem and networking I/O for Linux based on [`io_uring`][io_uring].
- A fallback implementation for other platforms based on [`Tokio`][tokio].
- A multi-thread runtime.

[io_uring]: https://unixism.net/loti/
[tokio]: https://github.com/tokio-rs/tokio

## Examples

```rust
use photonio::{fs::File, io::Write, io::WriteAt};

#[photonio::main]
async fn main() -> std::io::Result<()> {
    let mut file = File::create("hello.txt").await?;
    file.write(b"hello").await?;
    file.write_at(b"world", 5).await?;
    Ok(())
}
```

## Limitations

- Dropping an unfinished future for asynchronous filesystem or networking operations will result in a panic. However, this behavior might be change in the future.
- The current multi-thread runtime uses a naive round-robin fashion to schedule tasks. A work-stealing scheduler will be added in the future.