use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    hex: String,
}


impl Color {

    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() != 6 {
            return Err(format!("Invalid hex color length: expected 6 characters, got {}", hex.len()));
        }
        
        // Validate by parsing (ensures valid hex)
        u8::from_str_radix(&hex[0..2], 16).map_err(|e| format!("Invalid red component: {}", e))?;
        u8::from_str_radix(&hex[2..4], 16).map_err(|e| format!("Invalid green component: {}", e))?;
        u8::from_str_radix(&hex[4..6], 16).map_err(|e| format!("Invalid blue component: {}", e))?;
        
        Ok(Self {
            hex: format!("#{}", hex.to_uppercase())
        })
    }

    pub fn to_hex(&self) -> &str {
        &self.hex
    }
}