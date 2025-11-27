use clap::Parser;
use std::io::Error;
use tracing::{debug, error, info};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    //Path to file
    #[arg(short, long)]
    _type_: String,
    _addr_: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    if args._type_.contains("server") {
        info!("server started");
        let _ = server(args._addr_).await;
    } else if args._type_.as_str().contains("client") {
        let _ = client(args._addr_).await;
        info!("client started");
    } else {
        return;
    }
}
async fn server(_addr: String) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(_addr).await?;
    debug!("Listening on {}", listener.local_addr()?);
    loop {
        let (mut socket, addr) = listener.accept().await?;
        info!("New user connected: {addr}");
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        info!("User disconnected: {addr}");
                        return;
                    }
                    Ok(_n) => {
                        if let Ok(msg) = String::from_utf8(buf.clone()) {
                            info!("{addr}: {msg}");
                        }
                        buf.fill(0);
                    }
                    Err(e) => {
                        error!("Failed read from socket {}: {}", addr, e);
                        return;
                    }
                }
            }
        });
    }
}
async fn client(_addr: String) -> Result<(), Error> {
    let mut stream = TcpStream::connect(_addr).await?;
    loop {
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        let _ = stream.write(input.trim().as_bytes()).await;
    }
}
