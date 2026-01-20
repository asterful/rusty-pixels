pub mod canvas;
pub mod change;
pub mod color;
pub mod history;

use canvas::{Canvas, CanvasError};
use change::{Change, ChangeEvent};
use history::History;
use std::time::{SystemTime, UNIX_EPOCH};

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

        match &event {
            ChangeEvent::Paint { x, y, color } => {
                self.canvas.set_pixel(*x, *y, *color)?;
            }
            ChangeEvent::Resize { anchor, width, height } => {
                self.canvas.resize(*width, *height, *anchor)?;
            }
        }

        let change = Change {
            event,
            timestamp: self.get_current_timestamp(),
        };
        self.history.record_change(change, &self.canvas);

        Ok(())
    }

    /// Get the current Unix timestamp in milliseconds
    fn get_current_timestamp(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_millis() as u64
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
