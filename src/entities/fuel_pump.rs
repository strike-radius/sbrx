// entities/fuel_pump.rs

use crate::Fighter;
use piston_window::*;

pub struct FuelPump {
    pub x: f64,
    pub y: f64,
}

impl FuelPump {
    pub fn new(x: f64, y: f64) -> Self {
        FuelPump { x, y }
    }

    pub fn fuel_interaction(&self, fighter: &mut Fighter, texture: &G2dTexture) {
        let pump_center_x = self.x + texture.get_width() as f64 / 2.0;
        let pump_center_y = self.y + texture.get_height() as f64 / 2.0;
        let dx = fighter.x - pump_center_x;
        let dy = fighter.y - pump_center_y;

        // Interaction distance (150.0)
        if (dx * dx + dy * dy).sqrt() < 150.0 {
            // Refill fuel
            fighter.fuel = fighter.max_fuel;
            fighter
                .fuel_tanks
                .insert(fighter.fighter_type, fighter.fuel);
        }
    }

    pub fn draw(&self, context: Context, g: &mut G2d, texture: &G2dTexture) {
        image(texture, context.transform.trans(self.x, self.y), g);
    }
}
