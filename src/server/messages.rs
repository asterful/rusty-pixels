use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "chat")]
    Chat { content: String },
    
    #[serde(rename = "draw")]
    Draw { x: i32, y: i32, color: String },
    
    #[serde(rename = "clear")]
    Clear,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "chat")]
    Chat { 
        sender: String,
        content: String 
    },
    
    #[serde(rename = "draw")]
    Draw { 
        sender: String,
        x: i32, 
        y: i32, 
        color: String 
    },
    
    #[serde(rename = "clear")]
    Clear { 
        sender: String 
    },
    
    #[serde(rename = "error")]
    Error { 
        message: String 
    },
}
