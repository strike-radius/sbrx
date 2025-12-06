// utils/collision.rs

/// Checks if a point is close enough to a line segment to be considered a collision.
///
/// # Arguments
/// * `line_start_x`, `line_start_y` - Start position of the line
/// * `line_end_x`, `line_end_y` - End position of the line
/// * `point_x`, `point_y` - Position of the point to check
///
/// # Returns
/// `true` if the point is close enough to the line to be considered a collision
pub fn check_line_collision(
    line_start_x: f64,
    line_start_y: f64,
    line_end_x: f64,
    line_end_y: f64,
    point_x: f64,
    point_y: f64,
) -> bool {
    // Calculate the minimum distance from point to line segment
    let line_dx = line_end_x - line_start_x;
    let line_dy = line_end_y - line_start_y;
    let line_length = (line_dx * line_dx + line_dy * line_dy).sqrt();

    if line_length == 0.0 {
        return false;
    }

    // Calculate closest point on line
    let t = ((point_x - line_start_x) * line_dx + (point_y - line_start_y) * line_dy)
        / (line_length * line_length);

    // Clamp t to [0,1] to keep within line segment
    let t = t.max(0.0).min(1.0);

    let closest_x = line_start_x + t * line_dx;
    let closest_y = line_start_y + t * line_dy;

    // Calculate distance from point to closest point on line
    let dx = point_x - closest_x;
    let dy = point_y - closest_y;
    let distance = (dx * dx + dy * dy).sqrt();

    // Check if distance is within collision threshold
    distance < 50.0 // Adjust this value to change collision width
}
