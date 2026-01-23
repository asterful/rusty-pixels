use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    hex: String,  // Store as hex string internally for performance
}

#[allow(dead_code)]
impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            hex: format!("#{:02X}{:02X}{:02X}", r, g, b)
        }
    }

    pub fn black() -> Self {
        Self {
            hex: "#000000".to_string()
        }
    }

    pub fn white() -> Self {
        Self {
            hex: "#FFFFFF".to_string()
        }
    }

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
    
    // RGB component accessors if needed
    pub fn r(&self) -> u8 {
        u8::from_str_radix(&self.hex[1..3], 16).unwrap()
    }
    
    pub fn g(&self) -> u8 {
        u8::from_str_radix(&self.hex[3..5], 16).unwrap()
    }
    
    pub fn b(&self) -> u8 {
        u8::from_str_radix(&self.hex[5..7], 16).unwrap()
    }
}