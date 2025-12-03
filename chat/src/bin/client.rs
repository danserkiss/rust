use chat::{Message, Parser, error, info, rd_sock_to_string, wr_string_to_sock};
use tokio::net::TcpStream;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    // IP address and port to connect
    #[arg(short, long)]
    addr: String,
}

struct Client {
    addr: String,
}
impl Client {
    async fn connect(&self) {
        if let Ok(stream) = TcpStream::connect(self.addr.clone()).await {
            let (sock_read, sock_write) = stream.into_split();
            self.send_msg(sock_write).await;
            self.recv_msg(sock_read).await;
        } else {
            error!("cannot connect to server");
            return;
        }
    }
    async fn send_msg(&self, mut sock_write: tokio::net::tcp::OwnedWriteHalf) {
        tokio::spawn(async move {
            loop {
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
                let trim_input = input.trim();
                match wr_string_to_sock(&mut sock_write, trim_input.to_string()).await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("{e}");
                    }
                }
            }
        });
    }
    async fn recv_msg(&self, mut sock_read: tokio::net::tcp::OwnedReadHalf) {
        loop {
            if let Ok(json) = rd_sock_to_string(&mut sock_read).await {
                if let Ok(msg) = serde_json::from_str::<Message>(&json) {
                    println!("{}: {}", msg.addr, msg.msg);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("client started");

    let args = Args::parse();
    let client = Client { addr: args.addr };
    client.connect().await;
}
