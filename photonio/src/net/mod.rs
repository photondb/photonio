mod tcp;
pub use tcp::{TcpListener, TcpStream};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Shutdown {
    Both,
    Read,
    Write,
}
