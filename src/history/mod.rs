pub mod change;

use std::sync::Mutex;
use std::sync::Arc;

use crate::world::canvas::Canvas;
use crate::history::change::Change;
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc::UnboundedSender;


#[derive(Debug)]
pub enum RollbackError {
    IndexOutOfBounds {
        #[allow(dead_code)]
        target: usize,
        #[allow(dead_code)]
        max: usize,
    },
}


#[derive(Serialize, Deserialize, Clone)]
pub struct Snapshot {
    pub canvas: Canvas,
    pub change_count: usize,
}


pub struct History {
    tx: UnboundedSender<(Change, Option<Canvas>)>,
    conn: Arc<Mutex<rusqlite::Connection>>,
    snapshot_interval: usize,
    event_count: std::sync::atomic::AtomicUsize,
}


#[allow(dead_code)]
impl History {

    pub fn open<P: AsRef<std::path::Path>>(db_path: P, snapshot_interval: usize) -> Result<Self, rusqlite::Error> {
        let conn = rusqlite::Connection::open(db_path)?;

        // Enable WAL mode and foreign keys
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        // Executes schema.sql (tables are created only if they don't exist yet)
        const SCHEMA_SQL: &str = include_str!("sql/schema.sql");
        conn.execute_batch(SCHEMA_SQL)?;

        let conn_arc = Arc::new(Mutex::new(conn));

        let initial_count: usize = conn_arc
            .lock()
            .unwrap()
            .query_row("SELECT COALESCE(MAX(id), 0) FROM events", [], |row| row.get(0))
            .unwrap_or(0);

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<(Change, Option<Canvas>)>();

        let writer_conn = Arc::clone(&conn_arc);
        std::thread::spawn(move || {
            while let Some((_change, _snapshot)) = rx.blocking_recv() {
                //let mut conn = writer_conn.lock().unwrap();
                //let _ = Self::write_change_to_db(&mut conn, change, snapshot);
            }
        });

        Ok(Self {
            tx,
            conn: conn_arc,
            snapshot_interval,
            event_count: std::sync::atomic::AtomicUsize::new(initial_count),
        })
    }

    /* 
    /// Create a new history tracker with the specified snapshot interval
    pub fn new(snapshot_interval: usize, initial_canvas: &Canvas) -> Self {
        // Open an in-memory SQLite database for ephemeral use
        let history = Self::open(":memory:", snapshot_interval)
            .expect("Failed to create in-memory history database");

        // If you need to record an initial state or init event, 
        // you can handle it here or let your application initialization handle it.

        history
    }
    */

    /// Record a new change and create a snapshot if needed
    pub fn record_change(&mut self, change: Change, current_canvas: &Canvas) {
        self.changes.push(change);
        
        if self.changes.len() % self.snapshot_interval == 0 {
            let snapshot = Snapshot {
                canvas: current_canvas.clone(),
                change_count: self.changes.len(),
            };
            self.snapshots.push(snapshot);
        }
    }

    /// Get the current number of changes
    pub fn current_change_count(&self) -> usize {
        self.changes.len()
    }

    /// Get the latest snapshot before or at the given change index
    pub fn latest_snapshot_before(&self, change_index: usize) -> Option<&Snapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.change_count <= change_index)
            .max_by_key(|s| s.change_count)
    }

    /// Reconstruct a canvas from history by replaying all changes
    pub fn reconstruct_canvas(&self) -> Canvas {
        use crate::history::change::ChangeEvent;
        
        // Always start from the last snapshot (there's always at least one)
        let snapshot = self.snapshots.last().expect("History must have at least one snapshot");
        let mut canvas = snapshot.canvas.clone();
        
        // Replay changes since the snapshot
        for change in &self.changes[snapshot.change_count..] {
            match &change.event {
                ChangeEvent::Paint { x, y, color } => {
                    let _ = canvas.set_pixel(*x, *y, color.clone());
                }
                ChangeEvent::Resize { anchor, width, height } => {
                    let _ = canvas.resize(*width, *height, *anchor);
                }
                ChangeEvent::Init { .. } | ChangeEvent::Rollback { .. } => {
                    // Non-canvas-mutating events; ignore during replay
                }
            }
        }
        
        canvas
    }

    /// Rollback to a specific change index (destructive)
    /// Index is 0-based. Truncates all changes after target_index.
    pub fn rollback_to_index(&mut self, target_index: usize) -> Result<(), RollbackError> {
        if target_index >= self.changes.len() {
            return Err(RollbackError::IndexOutOfBounds {
                target: target_index,
                max: self.changes.len().saturating_sub(1),
            });
        }
        
        // Truncate changes to keep only up to and including target
        self.changes.truncate(target_index + 1);
        let new_change_count = self.changes.len();
        
        // Remove snapshots that are after the new change count
        self.snapshots.retain(|snapshot| snapshot.change_count <= new_change_count);
        
        // Ensure we always have at least the initial snapshot
        if self.snapshots.is_empty() {
            panic!("History must always have at least one snapshot");
        }
        
        Ok(())
    }
}
