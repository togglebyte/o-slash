use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc};

use crate::errors::Result;

mod connection;
mod db;

use connection::{Reader, Writer};

pub type Rx = mpsc::Receiver<String>;
pub type Tx = mpsc::Sender<String>;

pub type BcRx = broadcast::Receiver<String>;
pub type BcTx = broadcast::Sender<String>;

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
        let (bc_tx, bc_rx) = broadcast::channel(100);

        loop {
            eprintln!("{:?}", "ready to accept connection");

            if let Ok((stream, addr)) = listener.accept().await {
                let (reader, writer) = stream.into_split();

                let (tx, rx) = mpsc::channel(10);
                let reader = Reader::new(reader, addr, tx.clone());
                let writer = Writer::new(writer, rx);

                tokio::spawn(client_connected(reader, bc_tx.clone()));
                tokio::spawn(writer.run());
                tokio::spawn(broadcasting(bc_tx.subscribe(), tx));
            }
        }
    }

}

async fn broadcasting(mut bc: BcRx, tx: Tx) -> Option<()> {
    loop {
        let msg = bc.recv().await.ok()?;
        tx.send(msg).await;
    }

    Some(())
}

// New client connects
async fn client_connected<T: AsyncRead + Unpin>(mut connection: Reader<T>, bcast: BcTx) {
    let mut buffer = vec![0u8; 1024];

    loop {
        match connection.read(&mut buffer).await {
            Ok(0) => break, // connection closed
            Ok(n) => {
                let s = std::str::from_utf8(&buffer[..n]).unwrap();

                // Send to db task

                // TODO: Bcast (remove this once db task is done)
                bcast.send(s.into());

                eprintln!("> {}", s.trim());
            }
            Err(e) => {
                eprintln!("Read error: {:?}", e);
            }
        }

    }
}
