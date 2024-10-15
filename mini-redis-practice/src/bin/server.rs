use anyhow::Result;
use clap::Parser;
use std::{io, sync::Arc};
use thiserror::Error;
use tokio::{
    io::AsyncReadExt as _,
    net::{TcpListener, TcpStream},
    sync::Semaphore,
};
use tracing::{error, info};

#[tokio::main]
async fn main() {
    if let Err(e) = tracing_subscriber::fmt::try_init() {
        eprintln!("Failed to init tracing: {}", e);
    }

    let ServerArgs {
        port,
        limit_connections,
    } = ServerArgs::parse();

    match Server::new(port, limit_connections).await {
        Ok(server) => match server.run().await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error running server: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error creating server: {}", e);
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct ServerArgs {
    #[clap(short, long, default_value = "8080")]
    port: u32,

    #[clap(short, long, default_value = "250")]
    limit_connections: usize,
}

struct Server {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

#[derive(Error, Debug)]
enum ServerError {
    #[error("IO error: {0}")]
    IO(io::Error),
}

impl From<io::Error> for ServerError {
    fn from(e: io::Error) -> Self {
        ServerError::IO(e)
    }
}

impl Server {
    async fn new(port: u32, limit_connections: usize) -> Result<Self> {
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        Ok(Server {
            listener,
            limit_connections: Arc::new(Semaphore::new(limit_connections)),
        })
    }

    async fn run(&self) -> Result<()> {
        tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    println!("Received Ctrl-C signal");
                },
                result = self.handle_connection() => {
                    if let Err(e) = result {
                        eprintln!("Error handling connection: {}", e);
                    }
                }
        }

        Ok(())
    }

    async fn handle_connection(&self) -> Result<(), ServerError> {
        info!("accepting inbound connections");
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let (socket, address) = self.listener.accept().await?;
            info!("Accepted connection from {}", address);

            let mut handler = Handler::new(socket).unwrap();

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                };
                drop(permit);
            });
        }
    }
}

struct Handler {
    socket: TcpStream,
}

impl Handler {
    fn new(socket: TcpStream) -> Result<Self> {
        Ok(Handler { socket })
    }

    async fn run(&mut self) -> Result<()> {
        let mut buf = vec![0u8];
        self.socket.read_buf(&mut buf).await?;
        let requset = String::from_utf8(buf).expect("Our bytes should be valid utf8");
        println!("Received data: {:?}", requset);
        Ok(())
    }
}
