// utils/math.rs

use rand::Rng;

/// Safely generates a random number in the given range.
/// Logs a warning if min >= max and returns min in that case.
pub fn safe_gen_range(min: f64, max: f64, context: &str) -> f64 {
    if min >= max {
        println!(
            "WARNING: Empty range detected in {}: min={}, max={}",
            context, min, max
        );
        min // Return min value if range is empty to avoid crashing
    } else {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..max)
    }
}
