// config.rs

/// Enables or disables CPU entities in the game
pub const CPU_ENABLED: bool = true;

/// Enables or disables the Fog of War system.
pub const FOG_OF_WAR_ENABLED: bool = false; 

/// Directly initiates Arena Mode on program start when true
pub const ARENA_MODE: bool = false; // false = campaign

/// When enabled, reduces visual effects to improve performance || true = on || false = off
pub const PERFORMANCE_MODE: bool = true;

/// Game resolution constants
pub mod resolution {
    pub const WIDTH: f64 = 1920.0;
    pub const HEIGHT: f64 = 1080.0;
    pub const HORIZON_LINE: f64 = HEIGHT / 2.0;
}

/// Game / player boundaries    x: 5000.0 y: 3250.0
pub mod boundaries {
    pub const MIN_X: f64 = 0.0;
    pub const MAX_X: f64 = 5000.0;
    pub const MIN_Y: f64 = super::resolution::HORIZON_LINE;
    pub const MAX_Y: f64 = 3250.0;
}

/// Fighter movement constants
pub mod movement {
    pub const BIKE_SPEED: f64 = 650.0;
    pub const MOVEMENT_BUFFER_DURATION: f64 = 0.1;
    pub const RUSH_DURATION: f64 = 0.25;
    pub const RUSH_DISTANCE: f64 = 251.0;
    pub const ON_FOOT_HOLD_DURATION: f64 = 0.00;
    //pub const ON_FOOT_CONTINUOUS_SPEED: f64 = 150.0;
}

/// Gameplay constants
pub mod gameplay {
    pub const COLLISION_THRESHOLD: f64 = 75.0; // cpu strike/collision zone
    pub const BIKE_INTERACTION_DISTANCE: f64 = 125.0;
    pub const FIGHTER_JET_INTERACTION_DISTANCE: f64 = 150.0;
    pub const RAPTOR_NEST_INTERACTION_DISTANCE: f64 = 150.0;
    //pub const KINETIC_STRIKE_IMMUNITY_DURATION: f64 = 0.25;
    pub const WARNING_MESSAGE_DURATION: f64 = 3.0;
}
