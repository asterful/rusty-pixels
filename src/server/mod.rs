mod messages;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use messages::{ClientMessage, ServerMessage};
use crate::world::{World, color::Color};

type Clients = Arc<RwLock<HashMap<SocketAddr, tokio::sync::mpsc::UnboundedSender<Message>>>>;

pub struct Server {
    addr: String,
    clients: Clients,
    world: Arc<RwLock<World>>,
}

impl Server {
    pub fn new(addr: impl Into<String>) -> Self {
        let history = match crate::world::persistence::load_history() {
            Ok(history) if !history.snapshots.is_empty() => {
                println!("Loaded history from disk");
                history
            }
            _ => {
                println!("No valid history found, creating new world");
                let canvas = crate::world::canvas::Canvas::new(128, 128)
                    .expect("Failed to create canvas");
                crate::world::history::History::new(100, &canvas)
            }
        };
        
        let world = World::from(history);
        
        Self {
            addr: addr.into(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            world: Arc::new(RwLock::new(world)),
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("Server listening on {}", self.addr);

        // Spawn periodic save task (saves every 30 seconds)
        let world_for_save = self.world.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
            loop {
                interval.tick().await;
                let world_lock = world_for_save.read().await;
                if let Err(e) = crate::world::persistence::save_history(&world_lock.history) {
                    eprintln!("Failed to save history: {}", e);
                } else {
                    println!("History saved to disk");
                }
            }
        });

        while let Ok((stream, addr)) = listener.accept().await {
            println!("New connection from {}", addr);
            tokio::spawn(Self::handle_connection(
                stream, 
                addr, 
                self.clients.clone(), 
                self.world.clone()
            ));
        }

        Ok(())
    }

    async fn handle_connection(stream: TcpStream, addr: SocketAddr, clients: Clients, world: Arc<RwLock<World>>) {
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
        clients.write().await.insert(addr, tx.clone());

        // Send init message immediately
        let init_msg = {
            let world_lock = world.read().await;
            let width = world_lock.canvas.width();
            let height = world_lock.canvas.height();
            
            // Build the 2D board array
            let mut board = Vec::with_capacity(height);
            for y in 0..height {
                let mut row = Vec::with_capacity(width);
                for x in 0..width {
                    let color = world_lock.canvas.get_pixel(x, y)
                        .unwrap_or(Color::white());
                    row.push(color.to_hex());
                }
                board.push(row);
            }
            
            ServerMessage::Init {
                width,
                height,
                board,
                cooldown: 0,
            }
        };
        
        if let Ok(json) = serde_json::to_string(&init_msg) {
            if tx.send(Message::Text(json)).is_err() {
                eprintln!("Failed to send init message to {}", addr);
                clients.write().await.remove(&addr);
                return;
            }
        }

        // Spawn task to handle outgoing messages
        let send_task = tokio::spawn(async move {
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
                    Self::handle_json_message(&clients, &world, &text, addr).await;
                }
                Ok(Message::Binary(bin)) => {
                    Self::broadcast_message(&clients, Message::Binary(bin), addr).await;
                }
                Ok(Message::Close(_)) => {
                    println!("Client {} closed connection", addr);
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
        println!("Client {} disconnected", addr);
        send_task.abort();
        clients.write().await.remove(&addr);
    }

    async fn handle_json_message(clients: &Clients, world: &Arc<RwLock<World>>, text: &str, sender: SocketAddr) {
        let client_msg: ClientMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("Failed to parse JSON from {}: {} ({})", sender, e, text);
                return;
            }
        };

        match client_msg {
            ClientMessage::Paint { x, y, color } => {
                // Validate color format
                if !color.starts_with('#') || color.len() != 7 {
                    eprintln!("Invalid color format from {}: {}", sender, color);
                    return;
                }
                
                // Parse color
                let parsed_color = match Color::from_hex(&color) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Invalid color from {}: {} ({})", sender, color, e);
                        return;
                    }
                };
                
                // Apply the paint operation
                let result = {
                    let mut world_lock = world.write().await;
                    let paint_event = crate::world::change::ChangeEvent::Paint {
                        x,
                        y,
                        color: parsed_color,
                    };
                    world_lock.apply_event(paint_event)
                };
                
                match result {
                    Ok(_) => {
                        // Broadcast update to all clients (including sender)
                        let update_msg = ServerMessage::Update {
                            x,
                            y,
                            color: parsed_color.to_hex(),
                        };
                        
                        if let Ok(json) = serde_json::to_string(&update_msg) {
                            Self::broadcast_to_all(clients, Message::Text(json)).await;
                        }
                        
                        println!("Paint at ({}, {}) with color {} by {}", x, y, parsed_color.to_hex(), sender);
                    }
                    Err(e) => {
                        eprintln!("Failed to paint pixel from {}: {:?}", sender, e);
                    }
                }
            }
            ClientMessage::Ping => {
                // Respond with current client count
                let client_count = clients.read().await.len();
                let pong_msg = ServerMessage::Pong {
                    clients: client_count,
                };
                
                if let Ok(json) = serde_json::to_string(&pong_msg) {
                    if let Some(tx) = clients.read().await.get(&sender) {
                        tx.send(Message::Text(json)).ok();
                    }
                }
            }
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
    
    async fn broadcast_to_all(clients: &Clients, msg: Message) {
        let clients = clients.read().await;
        for tx in clients.values() {
            tx.send(msg.clone()).ok();
        }
    }
}
