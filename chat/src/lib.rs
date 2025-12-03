use anyhow::{Context, Ok, Result};
use bytes::{BufMut, BytesMut};
pub use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
pub use tracing::{debug, error, info};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub addr: String,
    pub msg: String,
}
pub async fn rd_sock_to_string(sock_read: &mut OwnedReadHalf) -> Result<String> {
    let len = sock_read.read_u32().await?;
    let mut json_buf = vec![0; len as usize];

    sock_read.read_exact(&mut json_buf).await?;
    String::from_utf8(json_buf).context("Recived not valid UTF-8")
}

pub async fn wr_string_to_sock(sock_write: &mut OwnedWriteHalf, json: String) -> Result<()> {
    let len = json.len() as u32;

    let mut data = BytesMut::new();
    data.put_u32(len);
    data.extend(json.as_bytes());
    sock_write.write_all(&data).await?;
    Ok(())
}
