use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::errors::Result;
mod connection;

use connection::{Reader, Writer};

// -----------------------------------------------------------------------------
//     - Server -
// -----------------------------------------------------------------------------
pub struct Server {
    host: &'static str
}

impl Server {
    pub fn new(host: &'static str) -> Self {
        Self {
            host
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        let listener = TcpListener::bind(self.host).await?;

        loop {
            eprintln!("{:?}", "ready to accept connection");

            if let Ok((stream, addr)) = listener.accept().await {
                let (reader, writer) = stream.into_split();
                let (tx, rx) = mpsc::channel(10);
                let reader = Reader::new(reader, addr, tx);
                let writer = Writer::new(writer, rx);

                tokio::spawn(client_connected(reader));
                tokio::spawn(writer.run());

            }
        }
    }

}

// New client connects
async fn client_connected<T: AsyncRead + Unpin>(mut connection: Reader<T>) {
    let mut buffer = vec![0u8; 1024];

    loop {
        match connection.read(&mut buffer).await {
            Ok(0) => break, // connection closed
            Ok(n) => {
                let s = std::str::from_utf8(&buffer[..n]).unwrap();

                // TODO: looks and smells of nonsense
                connection.send(s.into()).await;

                eprintln!("> {}", s.trim());
            }
            Err(e) => {
                eprintln!("Read error: {:?}", e);
            }
        }

    }
}
