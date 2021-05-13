use std::sync::Arc;
use super::{Rx, Tx};

pub async fn run_database(mut rx: Rx, broadcaster: Arc<Vec<Tx>>) -> Option<()>  {

    loop {
        let msg = rx.recv().await?;
        eprintln!("messege received: '{:?}'", msg);
        eprintln!("{:?}", "pretend store message");

        // Broadcast to everyone
        broadcaster.get(1).as_ref().unwrap().send(msg).await;
    }

    Some(())
}
