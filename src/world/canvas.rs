use super::color::Color;
use super::change::ResizeAnchor;
use super::palette::Palette;
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::sync::RwLock;


#[allow(dead_code)]
#[derive(Debug)]
pub enum CanvasError {
    OutOfBounds { width: usize, height: usize },
    InvalidDimensions { width: usize, height: usize },
}


#[derive(Clone, Serialize, Deserialize)]
pub struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,  // Store palette indices instead of colors
    #[serde(skip)]
    palette: Arc<RwLock<Palette>>,
}


impl Canvas {
    /// Create a new canvas with the given width and height, initialized to white
    pub fn new(width: usize, height: usize) -> Result<Self, CanvasError> {
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidDimensions { width, height });
        }

        let palette = Arc::new(RwLock::new(Palette::new()));
        // White is always index 0 in new palette
        
        Ok(Self {
            width,
            height,
            pixels: vec![0; width * height],  // 0 = white
            palette,
        })
    }
    
    #[allow(dead_code)]
    /// Create canvas with existing palette
    pub fn with_palette(width: usize, height: usize, palette: Arc<RwLock<Palette>>) -> Result<Self, CanvasError> {
        if width == 0 || height == 0 {
            return Err(CanvasError::InvalidDimensions { width, height });
        }
        
        Ok(Self {
            width,
            height,
            pixels: vec![0; width * height],
            palette,
        })
    }

    /// Get the width of the canvas
    pub fn width(&self) -> usize {
        self.width
    }

    /// Get the height of the canvas
    pub fn height(&self) -> usize {
        self.height
    }

    /// Set the color of the pixel at (x, y)
    pub fn set_pixel(&mut self, x: usize, y: usize, color: Color) -> Result<(), CanvasError> {
        if x >= self.width || y >= self.height {
            return Err(CanvasError::OutOfBounds {
                width: self.width,
                height: self.height,
            });
        }

        let index = y * self.width + x;
        let color_index = {
            let mut palette = self.palette.write().unwrap();
            palette.add_color(color.to_hex().to_string())
        };
        self.pixels[index] = color_index;
        Ok(())
    }

    #[allow(dead_code)]
    /// Get the color of the pixel at (x, y)
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, CanvasError> {
        if x >= self.width || y >= self.height {
            return Err(CanvasError::OutOfBounds {
                width: self.width,
                height: self.height,
            });
        }

        let index = y * self.width + x;
        let color_index = self.pixels[index];
        let palette = self.palette.read().unwrap();
        let hex = palette.get_color(color_index).unwrap_or("#FFFFFF");
        Color::from_hex(hex).map_err(|_| CanvasError::OutOfBounds { width: self.width, height: self.height })
    }

    /// Get direct access to the pixels slice (row-major order of palette indices)
    pub fn pixels(&self) -> &[u32] {
        &self.pixels
    }
    
    /// Get the palette
    pub fn palette(&self) -> Arc<RwLock<Palette>> {
        self.palette.clone()
    }

    /// Resize the canvas to new dimensions, anchoring the existing content
    pub fn resize(&mut self, new_width: usize, new_height: usize, anchor: ResizeAnchor) -> Result<(), CanvasError> {
        if new_width == 0 || new_height == 0 {
            return Err(CanvasError::InvalidDimensions {
                width: new_width,
                height: new_height,
            });
        }

        let mut new_pixels = vec![0u32; new_width * new_height];  // 0 = white/default

        let (offset_x, offset_y) = match anchor {
            ResizeAnchor::TopLeft => (0, 0),
            ResizeAnchor::TopRight => (new_width.saturating_sub(self.width), 0),
            ResizeAnchor::BottomLeft => (0, new_height.saturating_sub(self.height)),
            ResizeAnchor::BottomRight => (new_width.saturating_sub(self.width), new_height.saturating_sub(self.height)),
            ResizeAnchor::Center => (new_width.saturating_sub(self.width) / 2, new_height.saturating_sub(self.height) / 2),
        };

        for y in 0..self.height {
            for x in 0..self.width {
                let new_x = x + offset_x;
                let new_y = y + offset_y;

                if new_x < new_width && new_y < new_height {
                    let old_index = y * self.width + x;
                    let new_index = new_y * new_width + new_x;
                    new_pixels[new_index] = self.pixels[old_index];
                }
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.pixels = new_pixels;

        Ok(())
    }
}