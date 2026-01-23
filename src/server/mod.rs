mod messages;

use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::Message;
use futures_util::{StreamExt, SinkExt};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use std::collections::HashMap;
use messages::{ClientMessage, ServerMessage};
use crate::world::{World, color::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Admin,
    Player,
}

struct ClientInfo {
    sender: tokio::sync::mpsc::UnboundedSender<Message>,
    role: Role,
}

type Clients = Arc<RwLock<HashMap<SocketAddr, ClientInfo>>>;

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
                let canvas = crate::world::canvas::Canvas::new(
                    crate::env::default_canvas_width(),
                    crate::env::default_canvas_height()
                ).expect("Failed to create canvas");
                crate::world::history::History::new(
                    crate::env::default_snapshot_interval(),
                    &canvas
                )
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

        // Spawn periodic save task
        let world_for_save = self.world.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(crate::env::autosave_interval()));
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
        use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
        
        // Extract query parameters from the WebSocket handshake
        let query_params = Arc::new(Mutex::new(HashMap::new()));
        let params_for_callback = query_params.clone();
        
        let ws_stream = match tokio_tungstenite::accept_hdr_async(stream, |req: &Request, resp: Response| {
            // Extract query string from the request URI
            if let Some(query_string) = req.uri().query() {
                let mut params = HashMap::new();
                for param in query_string.split('&') {
                    if let Some((key, value)) = param.split_once('=') {
                        params.insert(
                            key.to_string(),
                            urlencoding::decode(value).unwrap_or_default().to_string()
                        );
                    }
                }
                if let Ok(mut p) = params_for_callback.lock() {
                    *p = params;
                }
            }
            Ok(resp)
        }).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("WebSocket handshake failed with {}: {}", addr, e);
                return;
            }
        };
        
        let query_params = query_params.lock().unwrap().clone();
        println!("Query parameters for {}: {:?}", addr, query_params);

        // Determine role based on auth parameter
        let role = match query_params.get("auth") {
            Some(token) if token == crate::env::admin_token() => Role::Admin,
            _ => Role::Player,
        };
        println!("Client {} connected as {:?}", addr, role);

        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        // Store the client with their role
        clients.write().await.insert(addr, ClientInfo {
            sender: tx.clone(),
            role,
        });

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
                    if let Some(client_info) = clients.read().await.get(&addr) {
                        client_info.sender.send(Message::Pong(data)).ok();
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
                    if let Some(client_info) = clients.read().await.get(&sender) {
                        client_info.sender.send(Message::Text(json)).ok();
                    }
                }
            }
        }
    }

    async fn broadcast_message(clients: &Clients, msg: Message, sender: SocketAddr) {
        let clients = clients.read().await;
        for (addr, client_info) in clients.iter() {
            if *addr != sender {
                client_info.sender.send(msg.clone()).ok();
            }
        }
    }
    
    async fn broadcast_to_all(clients: &Clients, msg: Message) {
        let clients = clients.read().await;
        for client_info in clients.values() {
            client_info.sender.send(msg.clone()).ok();
        }
    }
}
