use serde::{Deserialize, Serialize};

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "paint")]
    Paint { x: usize, y: usize, color: String },
    
    #[serde(rename = "ping")]
    Ping,
}

/// Messages sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "init")]
    Init {
        width: usize,
        height: usize,
        board: Vec<Vec<String>>,
        cooldown: u64,
    },
    
    #[serde(rename = "update")]
    Update {
        x: usize,
        y: usize,
        color: String,
    },
    
    #[serde(rename = "pong")]
    Pong {
        clients: usize,
    },
}
