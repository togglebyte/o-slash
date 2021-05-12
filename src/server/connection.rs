use std::net::SocketAddr;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use crate::errors::Result;

pub type Rx = mpsc::Receiver<String>;
pub type Tx = mpsc::Sender<String>;

// -----------------------------------------------------------------------------
//     - Reader -
// -----------------------------------------------------------------------------
pub(super) struct Reader<T: AsyncRead + Unpin> {
    reader: T,
    tx: Tx,
    addr: SocketAddr,
}

impl<T: AsyncRead + Unpin> Reader<T> {
    pub(super) fn new(reader: T, addr: SocketAddr, tx: Tx) -> Self {
        Self { reader, tx, addr }
    }

    pub(super) async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        Ok(self.reader.read(buf).await?)
    }

    pub(super) async fn send(&mut self, data: String) {
        self.tx.send(data).await;
    }
}

// -----------------------------------------------------------------------------
//     - Writer -
// -----------------------------------------------------------------------------
pub(super) struct Writer<T: AsyncWrite + Unpin> {
    writer: T,
    rx: Rx,
}

impl<T: AsyncWrite + Unpin> Writer<T> {
    pub(super) fn new(writer: T, rx: Rx ) -> Self {
        Self { writer, rx }
    }

    pub(super) async fn run(mut self) -> Result<()> {
        loop {
            if let Some(msg) = self.rx.recv().await {
                self.writer.write_all(msg.as_bytes()).await?;
            }
        }
    }
}
