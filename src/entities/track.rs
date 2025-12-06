// entities/track.rs

/// Represents the game track/level
pub struct Track {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Track {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Track {
            x,
            y,
            width,
            height,
        }
    }

    /// Create a track centered between the horizon line and the bottom of the screen
    pub fn centered(line_y: f64, screen_height: f64, width: f64, height: f64) -> Self {
        let x = -45.0; // Standard offset
        let y = (line_y + screen_height) / 2.0; // Halfway between horizon and bottom

        Track {
            x,
            y,
            width,
            height,
        }
    }
}
