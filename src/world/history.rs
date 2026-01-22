use super::canvas::Canvas;
use super::change::Change;
use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub canvas: Canvas,
    pub change_count: usize,
}


#[derive(Serialize, Deserialize)]
pub struct History {
    pub snapshots: Vec<Snapshot>,
    pub changes: Vec<Change>,
    snapshot_interval: usize,
}


#[allow(dead_code)]
impl History {
    /// Create a new history tracker with the specified snapshot interval
    pub fn new(snapshot_interval: usize, initial_canvas: &Canvas) -> Self {
        let initial_snapshot = Snapshot {
            canvas: initial_canvas.clone(),
            change_count: 0,
        };
        
        History {
            changes: Vec::new(),
            snapshots: vec![initial_snapshot],
            snapshot_interval,
        }
    }

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
        use super::change::ChangeEvent;
        
        // Always start from the last snapshot (there's always at least one)
        let snapshot = self.snapshots.last().expect("History must have at least one snapshot");
        let mut canvas = snapshot.canvas.clone();
        
        // Replay changes since the snapshot
        for change in &self.changes[snapshot.change_count..] {
            match &change.event {
                ChangeEvent::Paint { x, y, color } => {
                    let _ = canvas.set_pixel(*x, *y, *color);
                }
                ChangeEvent::Resize { anchor, width, height } => {
                    let _ = canvas.resize(*width, *height, *anchor);
                }
            }
        }
        
        canvas
    }
}