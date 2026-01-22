use std::sync::OnceLock;

static ADMIN_TOKEN: OnceLock<String> = OnceLock::new();
static DEFAULT_CANVAS_WIDTH: OnceLock<usize> = OnceLock::new();
static DEFAULT_CANVAS_HEIGHT: OnceLock<usize> = OnceLock::new();
static DEFAULT_SNAPSHOT_INTERVAL: OnceLock<usize> = OnceLock::new();
static PERSISTENCE_PATH: OnceLock<String> = OnceLock::new();

pub fn init() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    let token = std::env::var("ADMIN_TOKEN")
        .expect("ADMIN_TOKEN must be set in .env file");
    
    let width = std::env::var("DEFAULT_CANVAS_WIDTH")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);
    
    let height = std::env::var("DEFAULT_CANVAS_HEIGHT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(128);
    
    let snapshot_interval = std::env::var("DEFAULT_SNAPSHOT_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    
    let persistence_path = std::env::var("PERSISTENCE_PATH")
        .unwrap_or_else(|_| "history.bin".to_string());
    
    ADMIN_TOKEN.set(token).expect("Failed to set ADMIN_TOKEN");
    DEFAULT_CANVAS_WIDTH.set(width).expect("Failed to set DEFAULT_CANVAS_WIDTH");
    DEFAULT_CANVAS_HEIGHT.set(height).expect("Failed to set DEFAULT_CANVAS_HEIGHT");
    DEFAULT_SNAPSHOT_INTERVAL.set(snapshot_interval).expect("Failed to set DEFAULT_SNAPSHOT_INTERVAL");
    PERSISTENCE_PATH.set(persistence_path).expect("Failed to set PERSISTENCE_PATH");
    
    println!("Environment variables loaded");
    println!("Canvas size: {}x{}", width, height);
}

pub fn admin_token() -> &'static str {
    ADMIN_TOKEN.get().expect("Environment not initialized. Call env::init() first")
}

pub fn default_canvas_width() -> usize {
    *DEFAULT_CANVAS_WIDTH.get().expect("Environment not initialized. Call env::init() first")
}

pub fn default_canvas_height() -> usize {
    *DEFAULT_CANVAS_HEIGHT.get().expect("Environment not initialized. Call env::init() first")
}

pub fn default_snapshot_interval() -> usize {
    *DEFAULT_SNAPSHOT_INTERVAL.get().expect("Environment not initialized. Call env::init() first")
}

pub fn persistence_path() -> &'static str {
    PERSISTENCE_PATH.get().expect("Environment not initialized. Call env::init() first")
}
