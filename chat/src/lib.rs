use anyhow::Result;
use bytes::{BufMut, BytesMut};
pub use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
pub use tracing::{debug, error, info};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // IP address and port to connect/host
    #[arg(short, long)]
    pub addr: String,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub addr: String,
    pub msg: String,
}
pub async fn rd_sock_to_string(sock_read: &mut OwnedReadHalf) -> Result<String, &'static str> {
    match sock_read.read_u32().await {
        Ok(len) => {
            let mut json_buf = vec![0; len as usize];
            match sock_read.read_exact(&mut json_buf).await {
                Ok(_) => Ok(String::from_utf8(json_buf).unwrap()),
                Err(_) => Err("Failed read from socket"),
            }
        }
        Err(_) => Err("Failed read from socket"),
    }
}

pub async fn wr_string_to_sock(
    sock_write: &mut OwnedWriteHalf,
    json: String,
) -> Result<(), &'static str> {
    let len = json.len() as u32;

    let mut data = BytesMut::new();
    data.put_u32(len);
    data.extend(json.as_bytes());
    match sock_write.write_all(&data).await {
        Ok(_) => Ok(()),
        Err(_) => Err("Failed write to socket"),
    }
}
