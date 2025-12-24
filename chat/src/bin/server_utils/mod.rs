use chat::{Message, Parser, UserAddr, debug, error, info};
use std::result::Result::Ok;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, Sender};
pub mod axum;
pub mod clientset;
pub mod handler;
mod timeout;
use clientset::ClientSet;
use handler::Handler;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Port to host
    #[arg(short, long)]
    pub port: String,
}

pub struct Server {
    pub serv_addr: String,
    pub client_set: ClientSet,
    pub tx_rm_user: Sender<String>,
}

impl Server {
    pub fn new(serv_addr: String, set: ClientSet) -> Self {
        Server {
            serv_addr: serv_addr,
            client_set: set,
            tx_rm_user: mpsc::channel(1024).0,
        }
    }
    pub async fn listen_for_connections(mut self) {
        let Ok(listener) = TcpListener::bind(self.serv_addr).await else {
            error!("Error binding to addr");
            return;
        };

        if let Ok(local_addr) = listener.local_addr() {
            debug!("Listening on {}", local_addr);
        }
        let tx = broadcast::channel::<Message>(1024).0;

        while let Ok((socket, addr)) = listener.accept().await {
            info!("New user connected: {addr}");
            let addr_str = addr.to_string();
            let _ = tx.send(Message::UserJoined(UserAddr::new(addr_str.clone())));
            let client_set_cl = self.client_set.clone();
            let rx_admin = self.client_set.add_new_user(addr_str.clone()).await.1;
            let (sock_read, sock_write) = socket.into_split();
            Handler::run(
                addr_str.clone(),
                tx.clone(),
                sock_read,
                sock_write,
                client_set_cl.clone(),
                rx_admin,
            );
        }
    }
}
