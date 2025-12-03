use chat::{Message, Parser, debug, error, info, rd_sock_to_string, wr_string_to_sock};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    //Port to host
    #[arg(short, long)]
    port: String,
}

struct Server {
    serv_addr: String,
}
impl Server {
    async fn listen_for_connections(self) {
        if let Ok(listener) = TcpListener::bind(self.serv_addr).await {
            if let Ok(local_addr) = listener.local_addr() {
                debug!("Listening on {}", local_addr);
            }
            let (tx, _rx) = broadcast::channel::<String>(1024);

            loop {
                if let Ok((socket, addr)) = listener.accept().await {
                    info!("New user connected: {addr}");

                    let (sock_read, sock_write) = socket.into_split();
                    Server::uesr_handler(addr, sock_read, tx.clone()).await;
                    Server::broadcast_msg(tx.subscribe(), sock_write, addr.to_string()).await;
                } else {
                    error!("Error accepting connection");
                }
            }
        }
    }

    async fn broadcast_msg(mut rx: Receiver<String>, mut sock_write: OwnedWriteHalf, addr: String) {
        tokio::spawn(async move {
            loop {
                if let Ok(msg) = rx.recv().await {
                    if !msg.contains(&addr) {
                        match wr_string_to_sock(&mut sock_write, msg).await {
                            Ok(_) => {}
                            Err(_e) => {
                                return;
                            }
                        }
                    }
                }
            }
        });
    }

    async fn uesr_handler(addr: SocketAddr, mut sock_read: OwnedReadHalf, tx: Sender<String>) {
        tokio::spawn(async move {
            loop {
                if let Ok(msg) = rd_sock_to_string(&mut sock_read).await {
                    info!("{addr}: {}", msg);
                    let msg_st = Message {
                        addr: addr.to_string(),
                        msg: msg.clone(),
                    };

                    if let Ok(json) = serde_json::to_string(&msg_st) {
                        let _ = tx.send(json);
                    } else {
                        error!("Error converting <Message to json>");
                    }
                } else {
                    info!("User disconnected: {addr}");

                    return;
                }
            }
        });
    }
}

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
