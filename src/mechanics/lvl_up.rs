// src/mechanics/lvl_up.rs

pub const PROGRESSION_POINTS: [u32; 14] = [
    12,      // LVL 1 -> 2
    25,      // LVL 2 -> 3
    50,      // LVL 3 -> 4
    100,     // LVL 4 -> 5
    200,     // LVL 5 -> 6
    400,     // LVL 6 -> 7
    800,     // LVL 7 -> 8
    1_600,   // LVL 8 -> 9
    3_200,   // LVL 9 -> 10
    6_400,   // LVL 10 -> 11
    12_800,  // LVL 11 -> 12
    25_600,  // LVL 12 -> 13
    51_200,  // LVL 13 -> 14
    102_500, // LVL 14 -> 15 (effectively max level) was: u32::MAX,
];

/// Checks if a fighter has enough kills to level up based on their current level.
/// Returns the number of levels gained.
/// Modifies kills and level in place. Resets kills on level up.
pub fn check_for_level_up(kills: &mut u32, level: &mut u32) -> u32 {
    let mut levels_gained = 0;

    // Max level is PROGRESSION_POINTS.len() + 1.
    // The array index corresponds to `current_level - 1`.
    loop {
        if (*level as usize - 1) >= PROGRESSION_POINTS.len() {
            break; // Max level reached
        }

        let kills_needed = PROGRESSION_POINTS[*level as usize - 1];

        if *kills >= kills_needed {
            *kills -= kills_needed;
            *level += 1;
            levels_gained += 1;
        } else {
            break; // Not enough kills for the next level
        }
    }

    levels_gained
}
