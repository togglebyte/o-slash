use super::Rx;

pub async fn run(mut rx: Rx, id: usize) -> Option<()> {
    loop {
        let msg = rx.recv().await?;
        eprintln!("message received on {}: \"{}\"", id, msg);

    }

    Some(())
}
