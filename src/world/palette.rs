use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Palette {
    colors: Vec<String>,
    indices: HashMap<String, u32>,
}

impl Palette {
    pub fn new() -> Self {
        let mut palette = Self::default();
        // Start with white as index 0 (default color)
        palette.add_color("#FFFFFF".to_string());
        palette
    }
    
    /// Add a color to the palette if it doesn't exist, return its index
    pub fn add_color(&mut self, hex: String) -> u32 {
        if let Some(&index) = self.indices.get(&hex) {
            return index;
        }
        
        let index = self.colors.len() as u32;
        self.colors.push(hex.clone());
        self.indices.insert(hex, index);
        index
    }
    
    #[allow(dead_code)]
    /// Get the hex string for an index
    pub fn get_color(&self, index: u32) -> Option<&str> {
        self.colors.get(index as usize).map(|s| s.as_str())
    }
    
    #[allow(dead_code)]
    /// Get the index for a hex string
    pub fn get_index(&self, hex: &str) -> Option<u32> {
        self.indices.get(hex).copied()
    }
    
    /// Get all colors in the palette
    pub fn colors(&self) -> &[String] {
        &self.colors
    }
    
    #[allow(dead_code)]
    /// Get the number of colors in the palette
    pub fn len(&self) -> usize {
        self.colors.len()
    }
    
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.colors.is_empty()
    }
}
