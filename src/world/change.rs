use super::color::Color;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResizeAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ChangeEvent {
    Paint {
        x: usize,
        y: usize,
        color: Color,
    },
    Resize {
        anchor: ResizeAnchor,
        width: usize,
        height: usize,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Change {
    pub event: ChangeEvent,
    pub timestamp: u64,
}
