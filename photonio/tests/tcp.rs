use photonio::{
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    task,
};

#[photonio::test(env_logger = true)]
async fn server() {
    let server = TcpListener::bind("127.0.0.1:1234").await.unwrap();
    let server_addr = server.local_addr().unwrap();
    let mut tasks = Vec::new();
    let num_conns = 4;
    for i in 0..num_conns {
        let handle = task::spawn(send(server_addr, i));
        tasks.push(handle);
    }
    for _ in 0..num_conns {
        let (stream, _) = server.accept().await.unwrap();
        let handle = task::spawn(recv(stream));
        tasks.push(handle);
    }
    for task in tasks {
        task.await.unwrap();
    }
}

async fn send(addr: SocketAddr, byte: u8) {
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream.write(&[byte]).await.unwrap();
}

async fn recv(mut stream: TcpStream) {
    let mut buf = [0; 1];
    stream.read(&mut buf).await.unwrap();
    println!("recv: {}", buf[0]);
}
