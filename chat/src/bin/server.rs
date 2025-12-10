use chat::{Parser, info};
pub mod server_utils;
use crate::server_utils::{Args, Server};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("server started");

    let args = Args::parse();
    let mut addr = String::from("0.0.0.0:");
    addr.insert_str(8, &args.port);

    let serv = Server { serv_addr: addr };
    serv.listen_for_connections().await;
}
