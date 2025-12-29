pub mod client_utils;
use crate::client_utils::{Args, Client};
use chat::{Parser, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("client started");

    let args = Args::parse();
    let client = Client { addr: args.addr };
    client.connect().await;
}
