// Title: A simple Redis server
// Description: A simple Redis server that accepts connections and returns +OK for any input.
// it is a dummy redis server that accepts connections and returns +OK for any input.
// redis-cli -h 127.0.0.1 -p 6379，将尝试连接到本地主机的 6379 端口

use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096; // 4KB
                              // 通常情况下，我们会使用一个固定大小的缓冲区来读取数据，这个缓冲区的大小可以根据实际情况来调整，比如 4KB，8KB，16KB 等，这个缓冲区的大小不是越大越好，因为缓冲区越大，内存占用就越大，而且可能会导致内存碎片，所以需要根据实际情况来调整
                              // 这里是字节还是位？这里是字节，1 字节 = 8 位。1KB = 1024 字节，1MB = 1024KB，1GB = 1024MB

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init(); // 初始化日志库

    // build a listener
    let addr = "0.0.0.0:6379"; // 将监听所有的网络接口，端口是 6379
    let listener = TcpListener::bind(addr).await?;
    // println!("Listening on: {}", addr); // 建议使用 tracing 和 tracing-subscriber 来打印日志，这两个库是 tokio 的日志库，可以打印日志到控制台，文件，或者其他地方，需要通过 cargo add tracing 和 cargo add tracing-subscriber 来安装
    info!("DumyRedis: Listening on: {}", addr); // 使用 info! 宏来打印日志，这个宏是 tracing 提供的，可以打印日志到控制台，文件，或者其他地方
                                                // tracing 与 tracing-subscriber 是什么关系？
                                                // tracing 是一个日志库，提供了一些宏来打印日志，比如 info!，error!，debug! 等  tracing-subscriber 是一个日志输出库，提供了一些输出器，比如 fmt，file，env_logger 等，可以将日志输出到控制台，文件，环境变量等
                                                // Yes, you can think of TcpListener as a wrapper for handling incoming network requests over TCP.
                                                // It provides an asynchronous interface for listening to and accepting incoming TCP connections.

    loop {
        let (stream, raddr) = listener.accept().await?;
        info!("Accepted connection from: {}", raddr); // 打印客户端的地址 remote address

        tokio::spawn(async move {
            // process_redis_conn(stream).await.unwrap();
            if let Err(e) = process_redis_conn(stream, raddr).await {
                warn!("Error processing conn with {}: {:?}", raddr, e);
            }
        });
    }
}

async fn process_redis_conn(mut stream: tokio::net::TcpStream, raddr: SocketAddr) -> Result<()> {
    loop {
        // Wait for the socket to be readable
        stream.readable().await?;

        let mut buf = Vec::with_capacity(BUF_SIZE);

        // Try to read data, this may still fail with `WouldBlock`
        // if the readiness event is a false positive.
        match stream.try_read_buf(&mut buf) {
            Ok(0) => break, // EOF, end of file, 说明没有东西读了，直接退出循环
            Ok(n) => {
                info!("read {} bytes", n);
                // The from_utf8_lossy function in Rust is a method provided by the std::string::String module.
                // It is used to convert a slice of bytes (&[u8]) into a String, replacing any invalid UTF-8 sequences with the Unicode replacement character � (U+FFFD).
                let line = String::from_utf8_lossy(&buf);
                info!("read: {:?}", line);
                stream.write_all(b"+OK\r\n").await?; // 接收到任何数据，都返回 +OK\r\n
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // WouldBlock 是操作系统返回的错误，表示当前操作会阻塞，需要等待
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
    warn!("Connection {} closed", raddr);
    Ok(())
}

// io::ErrorKind::WouldBlock
// is a variant of the ErrorKind enum in Rust's standard library,
// specifically within the std::io module. It represents a non-blocking operation that would block if it were allowed to proceed.
// In the context of asynchronous or non-blocking I/O operations,
// WouldBlock is used to indicate that an operation cannot be completed immediately and would block the current thread if it were to wait for the operation to complete.
// Instead of blocking, the operation returns this error to signal that it should be retried later.
