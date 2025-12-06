// graphics/crater.rs

use piston_window::*;

/// Draws a crater with the specified properties
pub fn draw_crater(x: f64, y: f64, radius: f64, color: [f32; 4], context: Context, g: &mut G2d) {
    let num_periods = 50;
    let angle_increment = 2.0 * std::f64::consts::PI / num_periods as f64;

    for i in 0..num_periods {
        let angle = i as f64 * angle_increment;
        let period_x = x + radius * 1.0 * angle.cos();
        let period_y = y + radius * 0.75 * angle.sin();

        rectangle(color, [period_x, period_y, 2.0, 2.0], context.transform, g);
    }
}
