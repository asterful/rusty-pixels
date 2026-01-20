pub mod canvas;
pub mod change;
pub mod color;
pub mod history;

use canvas::{Canvas, CanvasError};
use change::{Change, ChangeEvent};
use history::History;

pub struct World {
    pub canvas: Canvas,
    pub history: History,
}

impl World {
    /// Create a new world with the given canvas dimensions and history snapshot interval
    pub fn new(width: usize, height: usize, snapshot_interval: usize) -> Result<Self, CanvasError> {
        let canvas = Canvas::new(width, height)?;
        let history = History::new(snapshot_interval);

        Ok(World { canvas, history })
    }

    /// Apply a change event to the world
    pub fn apply_event(&mut self, event: ChangeEvent) -> Result<(), CanvasError> {
        // Apply the event to the canvas
        match &event {
            ChangeEvent::Paint { x, y, color } => {
                self.canvas.set_pixel(*x, *y, *color)?;
            }
            ChangeEvent::Resize { anchor, width, height } => {
                self.canvas.resize(*width, *height, *anchor)?;
            }
        }

        // Record the change in history
        let change = Change {
            event,
            timestamp: self.get_current_timestamp(),
        };
        self.history.record_change(change, &self.canvas);

        Ok(())
    }

    /// Get the current timestamp (placeholder implementation)
    fn get_current_timestamp(&self) -> u64 {
        // In a real implementation, you would use something like:
        // std::time::SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        // For now, we'll use the change count as a simple timestamp
        self.history.current_change_count() as u64
    }

    /// Get the canvas dimensions
    pub fn dimensions(&self) -> (usize, usize) {
        (self.canvas.width(), self.canvas.height())
    }

    /// Get the total number of changes applied
    pub fn change_count(&self) -> usize {
        self.history.current_change_count()
    }
}
