use anyhow::Result;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Error;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
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

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Message {
    addr: String,
    msg: String,
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
    let clients_set = Arc::new(Mutex::new(HashMap::new()));

    debug!("Listening on {}", listener.local_addr()?);
    loop {
        let (socket, addr) = listener.accept().await?;
        let (mut sock_read, mut sock_wirte) = socket.into_split();

        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        clients_set.lock().await.insert(addr.clone(), tx.clone());
        let set_cl = clients_set.clone();

        info!("New user connected: {addr}");
        tokio::spawn(async move {
            let mut buf = BytesMut::new();
            loop {
                match sock_read.read_buf(&mut buf).await {
                    Ok(0) => {
                        info!("User disconnected: {addr}");
                        return;
                    }
                    Ok(_n) => {
                        let msg = String::from_utf8_lossy(&buf).to_string();
                        info!("{addr}: {msg}");
                        let msg_st = Message {
                            addr: addr.to_string(),
                            msg: msg.clone(),
                        };
                        for (a, t) in set_cl.lock().await.iter() {
                            if *a != addr {
                                let _ = t.send(msg_st.clone());
                            }
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
        tokio::spawn(async move {
            loop {
                if let Some(msg) = rx.recv().await {
                    let json = serde_json::to_string(&msg).unwrap();
                    let len = json.len() as u32;

                    let mut data = BytesMut::new();
                    data.put_u32(len);
                    data.extend(json.as_bytes());
                    let _ = sock_wirte.write_all(&data).await;
                }
            }
        });
    }
}
async fn client(_addr: String) -> Result<(), Error> {
    let stream = TcpStream::connect(_addr).await?;
    let (mut sock_read, mut sock_write) = stream.into_split();
    tokio::spawn(async move {
        loop {
            match sock_read.read_u32().await {
                Ok(0) => {
                    println!("Disconected");
                    return;
                }
                Ok(len) => {
                    let mut json_buf = vec![0u8; len as usize];
                    let _ = sock_read.read_exact(&mut json_buf).await;

                    let msg = serde_json::from_slice::<Message>(&json_buf).unwrap();
                    println!("{}: {}", msg.addr, msg.msg);
                }
                Err(_e) => {}
            }
        }
    });
    loop {
        let mut input = String::new();
        let _ = std::io::stdin().read_line(&mut input);
        let _ = sock_write.write(input.trim().as_bytes()).await;
    }
}
