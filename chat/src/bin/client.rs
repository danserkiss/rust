use chat::{Args, Message, Parser, error, info, rd_sock_to_string, wr_string_to_sock};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("client started");

    let args = Args::parse();
    if let Ok(stream) = TcpStream::connect(args.addr).await {
        let (mut sock_read, mut sock_write) = stream.into_split();
        tokio::spawn(async move {
            loop {
                let json = rd_sock_to_string(&mut sock_read).await;
                if json.is_ok() {
                    let msg = serde_json::from_str::<Message>(&json.unwrap()).unwrap();
                    println!("{}: {}", msg.addr, msg.msg);
                }
            }
        });
        loop {
            let mut input = String::new();
            let _ = std::io::stdin().read_line(&mut input);
            let trim_input = input.trim();
            match wr_string_to_sock(&mut sock_write, trim_input.to_string()).await {
                Ok(_) => {}
                Err(e) => {
                    error!(e);
                }
            }
        }
    } else {
        error!("cannot connect to server");
    }
}
