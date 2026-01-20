use super::color::Color;

#[derive(Debug, Clone, Copy)]
pub enum ResizeAnchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
}

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

pub struct Change {
    pub event: ChangeEvent,
    pub timestamp: u64,
}
