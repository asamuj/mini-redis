use anyhow::Result;
use clap::Parser;
use tokio::{io::AsyncWriteExt as _, net::TcpStream};

#[tokio::main]
async fn main() {
    let args = ClientArgs::parse();

    let res = Client::new(&args.addr).await;
    let client = match res {
        Ok(clinet) => clinet,
        Err(e) => {
            eprintln!("Failed to connect to server: {}", e);
            return;
        }
    };

    match args.cmd {
        Command::Get { key } => {
            // client.send(data).await.unwrap();
            println!("get key: {}", key);
        }
        Command::Set { key, val } => {
            println!("set key: {} val: {}", key, val);
            client.send(b"set key val").await.unwrap();
        }
    }
    println!("{:?}", args.addr);
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ClientArgs {
    #[clap(short, long, default_value = "127.0.0.1:8080")]
    addr: String,

    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Parser, Debug)]
enum Command {
    Set { key: String, val: String },
    Get { key: String },
}

struct Client {
    server_addr: String,
}

impl Client {
    async fn new(addr: &str) -> Result<Self> {
        Ok(Client {
            server_addr: addr.to_string(),
        })
    }

    async fn send(&self, data: &[u8]) -> Result<()> {
        let mut socket = TcpStream::connect(&self.server_addr).await?;
        socket.write_all(data).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client() {
        let client = Client::new("127.0.01:8080").await.unwrap();
        client.send(b"set key val").await.unwrap();

        
    }
}