use chat::Parser;
pub mod server_utils;
use crate::server_utils::clientset::ClientSet;
use crate::server_utils::{Args, Server, axum::AxumServer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let set = ClientSet::default();
    let set_cl = set.clone();

    let args = Args::parse();
    let mut addr = String::from("0.0.0.0:");
    addr.insert_str(8, &args.port);

    tokio::spawn(async move {
        AxumServer::start_http_server(set_cl).await;
    });

    let serv = Server::new(addr, set);
    serv.listen_for_connections().await;
}
