// src/map_system.rs - Updated MapSystem to store field colors

use rand::Rng;
use rand::SeedableRng;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FieldId(pub i32, pub i32);

pub struct MapSystem {
    pub current_plane_name: String,
    pub current_field_id: FieldId,
    origin_ground_color: [f32; 4],
    origin_sky_color: [f32; 4],
    // Store colors for each field to prevent flickering
    field_colors: HashMap<FieldId, ([f32; 4], [f32; 4])>, // (ground_color, sky_color)
}

impl MapSystem {
    pub fn new(plane_name: String, start_field_id: FieldId) -> Self {
        // Fixed colors for the origin field (FLATLINE_field.x[0]y[0])
        // Sky: black [0.0, 0.0, 0.0, 1.0]
        // Ground: mid-grey [0.5, 0.5, 0.5, 1.0] (this is the clear color)
        let origin_sky_color = [0.0, 0.0, 0.0, 1.0];
        let origin_ground_color = [0.5, 0.5, 0.5, 1.0];

        let mut field_colors = HashMap::new();
        // Store origin field colors
        field_colors.insert(FieldId(0, 0), (origin_ground_color, origin_sky_color));

        MapSystem {
            current_plane_name: plane_name,
            current_field_id: start_field_id,
            origin_ground_color,
            origin_sky_color,
            field_colors,
        }
    }

    /// Returns (ground_color, sky_color) for the current field.
    /// Colors are generated once per field and then stored to prevent flickering.
    pub fn get_current_field_colors(&mut self) -> ([f32; 4], [f32; 4]) {
        // Check if we already have colors stored for this field
        if let Some(&colors) = self.field_colors.get(&self.current_field_id) {
            return colors;
        }

        // If this is the origin field, return the fixed colors
        if self.current_field_id == FieldId(0, 0) {
            return (self.origin_ground_color, self.origin_sky_color);
        }

        // Generate new colors for this field and store them
        let mut rng = rand::rng();

        // Use field coordinates as seed for consistent colors per field
        // This ensures the same field always gets the same colors
        let seed =
            (self.current_field_id.0.wrapping_mul(31) + self.current_field_id.1).abs() as u64;
        let mut field_rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Ground: limit to 25%(darker) to 75%(lighter) on the grayscale scaling
        let ground_val = field_rng.gen_range(0.25..=0.75);
        let ground_color = [ground_val, ground_val, ground_val, 1.0];
        /*
                // Sky: limit to 0%(black) to 90%(light cap) on the grayscale scaling
                let sky_val = field_rng.gen_range(0.0..=0.90);
                let sky_color = [sky_val, sky_val, sky_val, 1.0];
        */

        // Sky: Always BLACK for Flatline
        let sky_color = [0.0, 0.0, 0.0, 1.0];

        let colors = (ground_color, sky_color);

        // Store the generated colors for this field
        self.field_colors.insert(self.current_field_id, colors);

        colors
    }

    pub fn transition_field_by_delta(&mut self, dx_fields: i32, dy_fields: i32) {
        self.current_field_id.0 += dx_fields;
        self.current_field_id.1 += dy_fields;
        println!(
            "Transitioned to plane '{}', field ID: ({}, {})",
            self.current_plane_name, self.current_field_id.0, self.current_field_id.1
        );
    }

    pub fn get_display_string(&self) -> String {
        format!(
            "{}_field.x[{}]y[{}]",
            self.current_plane_name, self.current_field_id.0, self.current_field_id.1
        )
    }
}

// Add this import to the top of main.rs if not already present:
// use rand::SeedableRng;
