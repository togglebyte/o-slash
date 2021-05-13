use std::sync::Arc;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio::sync::mpsc;

use crate::errors::Result;

mod connection;
mod db;
mod broadcaster;

use connection::{Reader, Writer};

pub type Rx = mpsc::Receiver<String>;
pub type Tx = mpsc::Sender<String>;


const TASK_COUNT: usize = 8;

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
        let mut b = Vec::new();
        

        // TODO: add every single connection to the broadcast list.
        // Broadcasters
        for id in 0..TASK_COUNT {
            let (broadcaster_tx, broadcaster_rx) = mpsc::channel(100);
            b.push(broadcaster_tx);
            broadcaster::run(broadcaster_rx, id);
        }

        let broadcast = Arc::new(b);
        // Start db
        let (db_tx, db_rx) = mpsc::channel(100);
        db::run_database(db_rx, broadcast.clone());

        loop {
            eprintln!("{:?}", "ready to accept connection");

            if let Ok((stream, addr)) = listener.accept().await {
                let (reader, writer) = stream.into_split();
                let (tx, rx) = mpsc::channel(10);
                let reader = Reader::new(reader, addr, tx.clone());
                let writer = Writer::new(writer, rx);

                tokio::spawn(client_connected(reader, broadcast.clone()));
                tokio::spawn(writer.run());
            }
        }
    }

}

// New client connects
async fn client_connected<T: AsyncRead + Unpin>(mut connection: Reader<T>, bcast: Arc<Vec<Tx>>) {
    let mut buffer = vec![0u8; 1024];

    loop {
        match connection.read(&mut buffer).await {
            Ok(0) => break, // connection closed
            Ok(n) => {
                let s = std::str::from_utf8(&buffer[..n]).unwrap();

                // Send to db task

                // TODO: Bcast (remove this once db task is done)
                bcast.get(0).as_ref().unwrap().send(s.into());


                eprintln!("> {}", s.trim());
            }
            Err(e) => {
                eprintln!("Read error: {:?}", e);
            }
        }

    }
}
