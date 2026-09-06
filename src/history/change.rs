use crate::world::color::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ResizeAnchor {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
    Center = 4,
}

impl ResizeAnchor {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            4 => Self::Center,
            _ => Self::Center,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeEvent {
    Init {
        width: usize,
        height: usize,
    },
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
    Rollback {
        target_event_id: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub event: ChangeEvent,
    pub timestamp: u64,
}