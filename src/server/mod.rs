mod messages;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use messages::{ClientMessage, ServerMessage};

type Clients = Arc<RwLock<HashMap<SocketAddr, tokio::sync::mpsc::UnboundedSender<Message>>>>;

pub struct Server {
    addr: String,
    clients: Clients,
}

impl Server {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            addr: addr.into(),
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;

        while let Ok((stream, addr)) = listener.accept().await {
            tokio::spawn(Self::handle_connection(stream, addr, self.clients.clone()));
        }

        Ok(())
    }

    async fn handle_connection(stream: TcpStream, addr: SocketAddr, clients: Clients) {
        let ws_stream = match accept_async(stream).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("WebSocket handshake failed with {}: {}", addr, e);
                return;
            }
        };

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Store the client
        clients.write().await.insert(addr, tx);

        // Spawn task to handle outgoing messages
        let mut send_task = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if ws_sender.send(msg).await.is_err() {
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    Self::handle_json_message(&clients, &text, addr).await;
                }
                Ok(Message::Binary(bin)) => {
                    Self::broadcast_message(&clients, Message::Binary(bin), addr).await;
                }
                Ok(Message::Close(_)) => {
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Some(tx) = clients.read().await.get(&addr) {
                        tx.send(Message::Pong(data)).ok();
                    }
                }
                Ok(Message::Pong(_)) => {}
                Ok(Message::Frame(_)) => {}
                Err(e) => {
                    eprintln!("WebSocket error for {}: {}", addr, e);
                    break;
                }
            }
        }

        // Clean up
        send_task.abort();
        clients.write().await.remove(&addr);
    }

    async fn handle_json_message(clients: &Clients, text: &str, sender: SocketAddr) {
        let client_msg: ClientMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {}", sender, e);
                let error_msg = ServerMessage::Error {
                    message: format!("Invalid JSON: {}", e),
                };
                if let Ok(json) = serde_json::to_string(&error_msg) {
                    if let Some(tx) = clients.read().await.get(&sender) {
                        tx.send(Message::Text(json)).ok();
                    }
                }
                return;
            }
        };

        let server_msg = match client_msg {
            ClientMessage::Chat { content } => ServerMessage::Chat {
                sender: sender.to_string(),
                content,
            },
            ClientMessage::Draw { x, y, color } => ServerMessage::Draw {
                sender: sender.to_string(),
                x,
                y,
                color,
            },
            ClientMessage::Clear => ServerMessage::Clear {
                sender: sender.to_string(),
            },
        };

        if let Ok(json) = serde_json::to_string(&server_msg) {
            Self::broadcast_message(clients, Message::Text(json), sender).await;
        }
    }

    async fn broadcast_message(clients: &Clients, msg: Message, sender: SocketAddr) {
        let clients = clients.read().await;
        for (addr, tx) in clients.iter() {
            if *addr != sender {
                tx.send(msg.clone()).ok();
            }
        }
    }
}
