// File: src/game_state.rs

// NOTE: Removed #[derive(PartialEq)] from GameState
pub enum GameState {
    TitleScreen,
    Playing,
    DeathScreen(DeathType),
    DeathScreenGroup {
        death_type: DeathType,
        downed_fighter_type: FighterType,
    },
    LoadingFirmament,                        // Added state for the loading screen
    FirmamentMode(Box<firmament_lib::Game>), // Added state for Firmament sub-game
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum DeathType {
	Crashed,
    Meteorite,
    GiantMantis,
    Rattlesnake,
    GiantRattlesnake,
    BloodIdol,
    VoidTempest,
    Raptor,
    TRex,
    FlyingSaucer,
    LightReaver,
    NightReaver,
    RazorFiend,
}

/// Ambient track state for [M] key cycling
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AmbientTrackState {
    Background,  // background_track.ogg (default)
    Crickets,    // crickets.ogg
    Muted,       // No ambient track (silence, just game SFX)
}

#[derive(PartialEq)]
pub enum RacerState {
    OnFoot,
    OnBike,
}

#[derive(PartialEq)]
pub enum MovementDirection {
    Forward,
    Backward,
    None,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum FighterType {
    Racer,
    Soldier,
    Hunter,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CombatMode {
    CloseCombat,
    Ranged,
    Balanced,
}
