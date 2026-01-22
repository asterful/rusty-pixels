use super::history::History;
use std::fs::File;
use std::io::{self, Read, Write};

const HISTORY_FILE: &str = "history.bin";

#[derive(Debug)]
pub enum PersistenceError {
    Io(io::Error),
    Serialization(bincode::Error),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersistenceError::Io(err) => write!(f, "IO error: {}", err),
            PersistenceError::Serialization(err) => write!(f, "Serialization error: {}", err),
        }
    }
}

impl std::error::Error for PersistenceError {}

impl From<io::Error> for PersistenceError {
    fn from(err: io::Error) -> Self {
        PersistenceError::Io(err)
    }
}

impl From<bincode::Error> for PersistenceError {
    fn from(err: bincode::Error) -> Self {
        PersistenceError::Serialization(err)
    }
}

/// Save history to disk using binary format with atomic write
pub fn save_history(history: &History) -> Result<(), PersistenceError> {
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
pub fn load_history() -> Result<History, PersistenceError> {
    let mut file = File::open(HISTORY_FILE)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    let history = bincode::deserialize(&buffer)?;
    Ok(history)
}

/// Load history from disk, or create a default if the file doesn't exist
pub fn load_or_create_history(snapshot_interval: usize) -> History {
    match load_history() {
        Ok(history) => {
            println!("Loaded history from disk");
            history
        }
        Err(e) => {
            match &e {
                PersistenceError::Io(io_err) if io_err.kind() == io::ErrorKind::NotFound => {
                    println!("History file not found, creating new history");
                }
                _ => {
                    eprintln!("Failed to load history: {}. Creating new history", e);
                }
            }
            History::new(snapshot_interval)
        }
    }
}
