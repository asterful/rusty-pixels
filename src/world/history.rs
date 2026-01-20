use super::canvas::Canvas;
use super::change::Change;


pub struct Snapshot {
    pub canvas: Canvas,
    pub change_count: usize,
}


pub struct History {
    pub snapshots: Vec<Snapshot>,
    pub changes: Vec<Change>,
    snapshot_interval: usize,
}


impl History {
    /// Create a new history tracker with the specified snapshot interval
    pub fn new(snapshot_interval: usize) -> Self {
        History {
            changes: Vec::new(),
            snapshots: Vec::new(),
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
}