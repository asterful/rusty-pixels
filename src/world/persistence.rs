use super::history::History;
use std::fs::File;
use std::io::{Read, Write};

const HISTORY_FILE: &str = "history.bin";

/// Save history to disk using binary format with atomic write
pub fn save_history(history: &History) -> Result<(), Box<dyn std::error::Error>> {
    const TEMP_FILE: &str = "history.bin.tmp";
    const BACKUP_FILE: &str = "history.bin.bak";
    
    let serialized = bincode::serialize(history)?;
    
    // Write to temporary file
    let mut file = File::create(TEMP_FILE)?;
    file.write_all(&serialized)?;
    file.sync_all()?; // Ensure data is written to disk
    drop(file);
    
    // Create backup of existing file if it exists
    if std::path::Path::new(HISTORY_FILE).exists() {
        std::fs::copy(HISTORY_FILE, BACKUP_FILE)?;
    }
    
    // Atomic rename
    std::fs::rename(TEMP_FILE, HISTORY_FILE)?;
    
    Ok(())
}

/// Load history from disk
pub fn load_history() -> Result<History, Box<dyn std::error::Error>> {
    let mut file = File::open(HISTORY_FILE)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let history = bincode::deserialize(&buffer)?;
    Ok(history)
}
