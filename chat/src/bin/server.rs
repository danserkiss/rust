use chat::{Args, Message, Parser, debug, error, info, rd_sock_to_string, wr_string_to_sock};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, mpsc};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("server started");

    let args = Args::parse();
    let mut addr = String::from("0.0.0.0:");
    addr.insert_str(8, &args.addr);
    if let Ok(listener) = TcpListener::bind(addr).await {
        let clients_set = Arc::new(Mutex::new(HashMap::new()));

        debug!("Listening on {}", listener.local_addr().unwrap());
        loop {
            let (socket, addr) = listener.accept().await.unwrap();
            let (mut sock_read, mut sock_write) = socket.into_split();

            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
            clients_set.lock().await.insert(addr.clone(), tx.clone());
            let set_cl = clients_set.clone();

            info!("New user connected: {addr}");
            tokio::spawn(async move {
                loop {
                    let msg = rd_sock_to_string(&mut sock_read).await;
                    if msg.is_ok() {
                        let msg_unwrap = msg.unwrap();
                        info!("{addr}: {}", msg_unwrap);
                        let msg_st = Message {
                            addr: addr.to_string(),
                            msg: msg_unwrap.clone(),
                        };
                        for (a, t) in set_cl.lock().await.iter() {
                            if *a != addr {
                                let _ = t.send(msg_st.clone());
                            }
                        }
                    } else {
                        info!("User disconnected: {addr}");
                        return;
                    }
                }
            });
            tokio::spawn(async move {
                loop {
                    if let Some(msg) = rx.recv().await {
                        let json = serde_json::to_string(&msg).unwrap();
                        match wr_string_to_sock(&mut sock_write, json).await {
                            Ok(_) => {}
                            Err(e) => {
                                error!(e);
                            }
                        }
                    }
                }
            });
        }
    }
}
