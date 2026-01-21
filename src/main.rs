mod server;
mod world;

use server::Server;

#[tokio::main]
async fn main() {
    println!("Starting Pixie WebSocket server...");
    
    let server = Server::new("127.0.0.1:8080");
    
    if let Err(e) = server.run().await {
        eprintln!("Server error: {}", e);
    }
}
