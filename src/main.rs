use o_slash::server::Server;

#[tokio::main]
async fn main() {
    let mut server = Server::new("127.0.0.1:5555");
    server.run().await;
}
