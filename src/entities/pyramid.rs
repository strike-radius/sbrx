// entities/pyramid.rs
/*
use crate::utils::math::safe_gen_range;
use piston_window::*;

/// Represents a pyramid obstacle in the game world
pub struct Pyramid {
    pub x: f64,
    pub y: f64,
    pub base_width: f64,
    pub height: f64,
}

impl Pyramid {
    /// Creates a new pyramid with the given position and dimensions
    pub fn new(x: f64, y: f64, base_width: f64, height: f64) -> Self {
        Pyramid {
            x,
            y,
            base_width,
            height,
        }
    }

    /// Creates a pyramid with random dimensions at the given position
    pub fn random(x: f64, y: f64) -> Self {
        Pyramid {
            x,
            y,
            base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
            height: safe_gen_range(30.0, 100.0, "Pyramid height"),
        }
    }

    /// Draws the pyramid on the screen
    pub fn draw(&self, context: Context, g: &mut G2d) {
        let base_left = self.x - self.base_width / 2.0;
        let base_right = self.x + self.base_width / 2.0;
        let peak = self.y - self.height;

        // Draw filled pyramid
        polygon(
            [0.0, 1.0, 0.0, 1.0], // Green color
            &[[base_left, self.y], [base_right, self.y], [self.x, peak]],
            context.transform,
            g,
        );

        // Draw outline
        line(
            [0.0, 0.0, 0.0, 1.0], // Black color
            1.0,                  // Line width
            [base_left, self.y, base_right, self.y],
            context.transform,
            g,
        );
        line(
            [0.0, 0.0, 0.0, 1.0],
            1.0,
            [base_left, self.y, self.x, peak],
            context.transform,
            g,
        );
        line(
            [0.0, 0.0, 0.0, 1.0],
            1.0,
            [base_right, self.y, self.x, peak],
            context.transform,
            g,
        );

        // Draw center line
        line(
            [0.0, 0.0, 0.0, 1.0],
            1.0,
            [self.x, peak, self.x, self.y],
            context.transform,
            g,
        );
    }
}

/// Helper function to generate a collection of border pyramids
pub fn generate_border_pyramids(
    left_border: f64,
    right_border: f64,
    bottom_border: f64,
    middle_line: f64,
) -> Vec<Pyramid> {
    let mut pyramids: Vec<Pyramid> = Vec::new();

    // Left border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_y = middle_line
            + safe_gen_range(
                i as f64 * 30.0,
                (i as f64 + 1.0) * 60.0,
                "Left border y pos",
            );

        // Make sure we stay within the vertical range
        if pyramid_y < bottom_border + 100.0 {
            pyramids.push(Pyramid {
                x: left_border - safe_gen_range(5.0, 40.0, "Left pyramid offset"), // Place them outside
                y: pyramid_y,
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Right border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_y = middle_line
            + safe_gen_range(
                i as f64 * 30.0,
                (i as f64 + 1.0) * 60.0,
                "Right border y pos",
            );

        // Make sure we stay within the vertical range
        if pyramid_y < bottom_border + 100.0 {
            pyramids.push(Pyramid {
                x: right_border + safe_gen_range(5.0, 40.0, "Right pyramid offset"), // Place them outside
                y: pyramid_y,
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Bottom border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_x = left_border
            + safe_gen_range(
                i as f64 * 50.0,
                (i as f64 + 1.0) * 60.0,
                "Bottom border x pos",
            );

        // Make sure we stay within the horizontal range
        if pyramid_x < right_border + 100.0 {
            pyramids.push(Pyramid {
                x: pyramid_x,
                y: bottom_border + safe_gen_range(5.0, 40.0, "Bottom pyramid offset"), // Place them outside
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Add a few random pyramids in the playable area for consistency with the original design
    for _ in 0..10 {
        let pyramid_x = safe_gen_range(50.0, 1870.0, "Pyramid x");
        let pyramid_base_width = safe_gen_range(25.0, 100.0, "Pyramid width");
        let pyramid_height = safe_gen_range(25.0, 100.0, "Pyramid height");

        pyramids.push(Pyramid {
            x: pyramid_x,
            y: 1080.0 / 2.0 + 20.0, // Just below the middle line
            base_width: pyramid_base_width,
            height: pyramid_height,
        });
    }

    pyramids
}

/*
// This code has been moved to entities/pyramid.rs, and can be called using the generate_border_pyramids function
// PYRAMIDS
    // Create border pyramids for visual boundaries
    let mut pyramids: Vec<Pyramid> = Vec::new();

    // Define the map boundaries
    let left_border = 0.0;
    let right_border = 5000.0;
    let bottom_border = 3250.0;
    let middle_line = line_y; // The horizontal line dividing the map (screen_height / 2.0)

    // Left border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_y = middle_line + safe_gen_range(i as f64 * 30.0, (i as f64 + 1.0) * 60.0, "Left border y pos");

        // Make sure we stay within the vertical range
        if pyramid_y < bottom_border + 100.0 {
            pyramids.push(Pyramid {
                x: left_border - safe_gen_range(5.0, 40.0, "Left pyramid offset"), // Place them outside
                y: pyramid_y,
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Right border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_y = middle_line + safe_gen_range(i as f64 * 30.0, (i as f64 + 1.0) * 60.0, "Right border y pos");

        // Make sure we stay within the vertical range
        if pyramid_y < bottom_border + 100.0 {
            pyramids.push(Pyramid {
                x: right_border + safe_gen_range(5.0, 40.0, "Right pyramid offset"), // Place them outside
                y: pyramid_y,
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Bottom border pyramids
    for i in 0..10 {
        // Random spacing for more natural appearance
        let pyramid_x = left_border + safe_gen_range(i as f64 * 50.0, (i as f64 + 1.0) * 60.0, "Bottom border x pos");

        // Make sure we stay within the horizontal range
        if pyramid_x < right_border + 100.0 {
            pyramids.push(Pyramid {
                x: pyramid_x,
                y: bottom_border + safe_gen_range(5.0, 40.0, "Bottom pyramid offset"), // Place them outside
                base_width: safe_gen_range(25.0, 80.0, "Pyramid width"),
                height: safe_gen_range(30.0, 100.0, "Pyramid height"),
            });
        }
    }

    // Add a few random pyramids in the playable area for consistency with the original design
    for _ in 0..10 {
        let pyramid_x = safe_gen_range(50.0, 1870.0, "Pyramid x");
        let pyramid_base_width = safe_gen_range(25.0, 100.0, "Pyramid width");
        let pyramid_height = safe_gen_range(25.0, 100.0, "Pyramid height");

        pyramids.push(Pyramid {
            x: pyramid_x,
            y: 1080.0 / 2.0 + 20.0, // Just below the middle line
            base_width: pyramid_base_width,
            height: pyramid_height,
        });
    }
*/

*/