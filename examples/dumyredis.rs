use std::net::SocketAddr;

use anyhow::Result;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpListener,
};
use tracing::{info, warn};

const BUF_SIZE: usize = 4096;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init(); // 初始化日志库

    // build a listener
    let addr = "0.0.0.0:6379";
    let listener = TcpListener::bind(addr).await?;
    // println!("Listening on: {}", addr); // 建议使用 tracing 和 tracing-subscriber 来打印日志，这两个库是 tokio 的日志库，可以打印日志到控制台，文件，或者其他地方，需要通过 cargo add tracing 和 cargo add tracing-subscriber 来安装
    info!("DumyRedis: Listening on: {}", addr); // 使用 info! 宏来打印日志，这个宏是 tracing 提供的，可以打印日志到控制台，文件，或者其他地方
                                                // tracing 与 tracing-subscriber 是什么关系？
                                                // tracing 是一个日志库，提供了一些宏来打印日志，比如 info!，error!，debug! 等  tracing-subscriber 是一个日志输出库，提供了一些输出器，比如 fmt，file，env_logger 等，可以将日志输出到控制台，文件，环境变量等

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
