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
