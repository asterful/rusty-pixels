use super::history::History;
use std::fs::File;
use std::io::{Read, Write};

/// Save history to disk using binary format with atomic write
pub fn save_history(history: &History) -> Result<(), Box<dyn std::error::Error>> {
    let history_file = crate::env::persistence_path();
    let temp_file = format!("{}.tmp", history_file);
    let backup_file = format!("{}.bak", history_file);
    
    let serialized = bincode::serialize(history)?;
    
    // Write to temporary file
    let mut file = File::create(&temp_file)?;
    file.write_all(&serialized)?;
    file.sync_all()?; // Ensure data is written to disk
    drop(file);
    
    // Create backup of existing file if it exists
    if std::path::Path::new(history_file).exists() {
        std::fs::copy(history_file, &backup_file)?;
    }
    
    // Atomic rename
    std::fs::rename(&temp_file, history_file)?;
    
    Ok(())
}

/// Load history from disk
pub fn load_history() -> Result<History, Box<dyn std::error::Error>> {
    let history_file = crate::env::persistence_path();
    let mut file = File::open(history_file)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let history = bincode::deserialize(&buffer)?;
    Ok(history)
}
