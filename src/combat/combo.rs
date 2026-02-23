// File: src/combat/combo.rs

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StrikeTimerState {
    Timer1,
    Timer2,
    Timer3,
}

pub struct StrikeResult {
    pub damage_multiplier: f64,
    pub knockback: bool,
    pub knockback_force: f64,
    pub apply_stun: bool,
    pub is_combo_finisher: bool,
    pub finisher_slash_count: u32,
    pub finisher_hit_count: u32,
}

pub struct ComboSystem {
    state: StrikeTimerState,
    pub timer: f64, // Time remaining in the 0.65s window
    strike_count: u32,
    last_combo_strike_timer: f64, // For visual effect duration
    melee_cooldown: f64,          // After Timer3 combo
    pub is_in_rest_period: bool,  // Flag to check if we are waiting for the timer to run out
    pub is_combo3_stun_disabled: bool,
	pub racer_combo_hit_connected: bool,
    // may need later
    //pub is_combo5_stun_disabled: bool,
}

impl ComboSystem {
    // --- Constants for Combo Tiers ---
    const STRIKE_TIMER_DURATION: f64 = 0.65;
    const POST_COMBO_3_COOLDOWN: f64 = 0.25;
    const COMBO_VISUAL_DURATION: f64 = 0.25; // How long the green slash appears
    const ENHANCED_KNOCKBACK_MULTIPLIER: f64 = 2.25; // 125%

    // Timer 1 (2-hit)
    const T1_STRIKE_ZONE: f64 = 0.20;
    const T1_STRIKES_ACCEPTED: u32 = 2;
    const T1_BASIC_DMG_MULT: f64 = 1.0; // 100% damage
    const T1_COMBO_DMG_MULT: f64 = 1.25;
    const T1_DAMAGE_REDUCTION: f64 = 0.50;

    // Timer 2 (3-hit)
    const T2_STRIKE_ZONE: f64 = 0.40;
    const T2_STRIKES_ACCEPTED: u32 = 3;
    const T2_BASIC_DMG_MULT: f64 = 1.25;
    const T2_COMBO_DMG_MULT: f64 = 1.75;
    const T2_DAMAGE_REDUCTION: f64 = 0.75;

    // Timer 3 (5-hit)
    const T3_STRIKE_ZONE: f64 = 1.25;
    const T3_STRIKES_ACCEPTED: u32 = 5;
    const T3_BASIC_DMG_MULT: f64 = 2.0;
    const T3_COMBO_DMG_MULT: f64 = 2.50;
    const T3_DAMAGE_REDUCTION: f64 = 0.95;

    pub fn new() -> Self {
        Self {
            state: StrikeTimerState::Timer1,
            timer: 0.0,
            strike_count: 0,
            last_combo_strike_timer: 0.0,
            melee_cooldown: 0.0,
            is_in_rest_period: false,
            is_combo3_stun_disabled: false,
			racer_combo_hit_connected: false,
            // may need later
            //is_combo5_stun_disabled: false,
        }
    }

    /// Called by a kinetic strike to begin the combo timer sequence.
    /// This sets the system as if the 2-hit combo just finished, opening the window
    /// for the 3-hit combo timer.
    pub fn start_timer_after_kinetic_strike(&mut self) {
        self.state = StrikeTimerState::Timer1; // We are "finishing" Timer1
        self.timer = Self::STRIKE_TIMER_DURATION; // Start the full timer
        self.strike_count = Self::T1_STRIKES_ACCEPTED; // Set strike count to max for the tier
        self.is_in_rest_period = true; // Enter the rest period, which will transition to Timer2 on expiry
        self.melee_cooldown = 0.0; // Ensure no cooldown prevents the next combo
        self.last_combo_strike_timer = 0.0; // Don't show a green slash for this
    }

    /// Resets the combo system to its initial state. Called on pause/unpause.
    pub fn reset(&mut self) {
        self.state = StrikeTimerState::Timer1;
        self.timer = 0.0;
        self.strike_count = 0;
        self.last_combo_strike_timer = 0.0;
        self.melee_cooldown = 0.0;
        self.is_in_rest_period = false;
        self.is_combo3_stun_disabled = false;
		self.racer_combo_hit_connected = false;
    }

    pub fn update(&mut self, dt: f64) {
        if self.melee_cooldown > 0.0 {
            self.melee_cooldown -= dt;
        }
        if self.last_combo_strike_timer > 0.0 {
            self.last_combo_strike_timer -= dt;
        }

        if self.timer > 0.0 {
            self.timer -= dt;

            if self.timer <= 0.0 {
                // Timer expired. This means a successful rest period completed.
                if self.is_in_rest_period {
                    self.is_in_rest_period = false;
                    self.strike_count = 0;
                    // Progress to the next state
                    // Default progression - will be overridden by fighter-specific logic in main.rs
                    self.state = match self.state {
                        StrikeTimerState::Timer1 => StrikeTimerState::Timer2,
                        StrikeTimerState::Timer2 => StrikeTimerState::Timer3,
                        StrikeTimerState::Timer3 => {
                            self.melee_cooldown = Self::POST_COMBO_3_COOLDOWN;
                            StrikeTimerState::Timer1 // Loop back
                        }
                    };
                } else {
                    // Timer expired but we weren't in a rest period (e.g., player hit once then stopped).
                    // This is a failed combo. Reset.
                    self.reset_to_timer1();
                }
            }
        }
    }

    pub fn handle_strike(&mut self) -> Option<StrikeResult> {
        if self.melee_cooldown > 0.0 {
            return None;
        }

        let (zone_end, accepted, basic_mult, combo_mult) = match self.state {
            StrikeTimerState::Timer1 => (
                Self::T1_STRIKE_ZONE,
                Self::T1_STRIKES_ACCEPTED,
                Self::T1_BASIC_DMG_MULT,
                Self::T1_COMBO_DMG_MULT,
            ),
            StrikeTimerState::Timer2 => (
                Self::T2_STRIKE_ZONE,
                Self::T2_STRIKES_ACCEPTED,
                Self::T2_BASIC_DMG_MULT,
                Self::T2_COMBO_DMG_MULT,
            ),
            StrikeTimerState::Timer3 => (
                Self::T3_STRIKE_ZONE,
                Self::T3_STRIKES_ACCEPTED,
                Self::T3_BASIC_DMG_MULT,
                Self::T3_COMBO_DMG_MULT,
            ),
        };

        if self.timer > 0.0 {
            let elapsed_time = Self::STRIKE_TIMER_DURATION - self.timer;
            if elapsed_time <= zone_end {
                // In Strike Zone
                self.is_in_rest_period = false;
                self.strike_count += 1;

                if self.strike_count > accepted {
                    // Exceeded strike count, RESET
                    self.reset_to_timer1_and_strike();
                    return Some(StrikeResult {
                        damage_multiplier: Self::T1_BASIC_DMG_MULT,
                        knockback: false,
                        knockback_force: 1000.0,
                        apply_stun: false,
                        is_combo_finisher: false,
                        finisher_slash_count: 1,
                        finisher_hit_count: 0,
                    });
                }

                if self.strike_count == accepted {
                    // COMBO FINISHER!
                    self.is_in_rest_period = true; // Now we wait for the timer to expire naturally
                    self.last_combo_strike_timer = Self::COMBO_VISUAL_DURATION;

                    let (slash_count, knockback, knockback_force, apply_stun) = match self.state {
                        StrikeTimerState::Timer1 => (1, true, 250.0, false), // 2-hit: knockback, no stun
                        StrikeTimerState::Timer2 => {
                            // 3-hit finisher: Always knockback, but only stun if allowed.
                            let can_stun = !self.is_combo3_stun_disabled;
                            (2, true, 1000.0, can_stun)
                        }
                        StrikeTimerState::Timer3 => {
                            // 5-hit finisher
                            self.is_combo3_stun_disabled = false;
                            // Always knockback and stun.
                            //(3, true, true)
                            // Always knockback with enhanced force, but no stun.
                            (3, true, 1000.0 * Self::ENHANCED_KNOCKBACK_MULTIPLIER, false)
                        }
                    };

                    return Some(StrikeResult {
                        damage_multiplier: combo_mult,
                        knockback,
                        knockback_force,
                        apply_stun,
                        is_combo_finisher: true,
                        finisher_slash_count: slash_count,
                        finisher_hit_count: accepted,
                    });
                } else {
                    // Basic strike within a combo sequence
                    return Some(StrikeResult {
                        damage_multiplier: basic_mult,
                        knockback: false,
                        knockback_force: 1000.0,
                        apply_stun: false,
                        is_combo_finisher: false,
                        finisher_slash_count: 1,
                        finisher_hit_count: 0,
                    });
                }
            } else {
                // In Rest Zone
                self.reset_to_timer1_and_strike();
                return Some(StrikeResult {
                    damage_multiplier: Self::T1_BASIC_DMG_MULT,
                    knockback: false,
                    knockback_force: 1000.0,
                    apply_stun: false,
                    is_combo_finisher: false,
                    finisher_slash_count: 1,
                    finisher_hit_count: 0,
                });
            }
        } else {
            // Idle, this is the first strike
            self.timer = Self::STRIKE_TIMER_DURATION;
            self.strike_count = 1;
            self.is_in_rest_period = false;
            return Some(StrikeResult {
                damage_multiplier: basic_mult,
                knockback: false,
                knockback_force: 1000.0,
                apply_stun: false,
                is_combo_finisher: false,
                finisher_slash_count: 1,
                finisher_hit_count: 0,
            });
        }
    }

    // Fighter-specific strike handling that respects their combo limitations
    pub fn handle_strike_for_fighter(
        &mut self,
        fighter_type: crate::game_state::FighterType,
    ) -> Option<StrikeResult> {
        use crate::game_state::FighterType;

        // Check if this fighter type can access the current timer state
        let can_use_current_state = match (fighter_type, self.state) {
            (FighterType::Raptor, StrikeTimerState::Timer2) => false, // raptor can't use 3-hit
            (FighterType::Raptor, StrikeTimerState::Timer3) => false, // raptor can't use 5-hit
            (FighterType::Soldier, StrikeTimerState::Timer3) => false, // Soldier can't use 5-hit
            _ => true, // All other combinations are allowed
        };

        if !can_use_current_state {
            // Force reset to Timer1 if trying to access forbidden combo
            self.reset_to_timer1_and_strike();
            return Some(StrikeResult {
                damage_multiplier: Self::T1_BASIC_DMG_MULT,
                knockback: false,
                knockback_force: 1000.0,
                apply_stun: false,
                is_combo_finisher: false,
                finisher_slash_count: if fighter_type == FighterType::Raptor {
                    3
                } else {
                    1
                },
                finisher_hit_count: 0,
            });
        }

        // Call regular handle_strike but modify result for raptor
        if let Some(mut result) = self.handle_strike() {
            if fighter_type == FighterType::Raptor {
                // raptor always gets triple slash visual for basic and 2-hit combo strikes
                if !result.is_combo_finisher || result.finisher_hit_count == 2 {
                    result.finisher_slash_count = 3;
                }
            }
            Some(result)
        } else {
            None
        }
    }

    fn reset_to_timer1(&mut self) {
        self.state = StrikeTimerState::Timer1;
        self.timer = 0.0;
        self.strike_count = 0;
        self.is_in_rest_period = false;
    }

    fn reset_to_timer1_and_strike(&mut self) {
        self.state = StrikeTimerState::Timer1;
        self.timer = Self::STRIKE_TIMER_DURATION;
        self.strike_count = 1;
        self.is_in_rest_period = false;
    }

    pub fn is_combo_strike_active(&self) -> bool {
        self.last_combo_strike_timer > 0.0
    }

    pub fn get_damage_intake_multiplier(&self) -> f64 {
        // Damage reduction is only active if a combo is in progress (timer is running)
        // and we are not in a rest period waiting for the timer to expire.
        if self.timer > 0.0 && !self.is_in_rest_period {
            match self.state {
                StrikeTimerState::Timer1 => 1.0 - Self::T1_DAMAGE_REDUCTION, // 0.75
                StrikeTimerState::Timer2 => 1.0 - Self::T2_DAMAGE_REDUCTION, // 0.50
                StrikeTimerState::Timer3 => 1.0 - Self::T3_DAMAGE_REDUCTION, // 0.25
            }
        } else {
            // No reduction if no combo is active
            1.0
        }
    }

    // Fighter-specific state progression
    pub fn progress_state_for_fighter(&mut self, fighter_type: crate::game_state::FighterType) {
        use crate::game_state::FighterType;

        self.state = match (self.state, fighter_type) {
            (StrikeTimerState::Timer1, FighterType::Raptor) => {
                // raptor can only do 2-hit combos, so reset after Timer1
                self.melee_cooldown = Self::POST_COMBO_3_COOLDOWN;
                StrikeTimerState::Timer1
            }
            (StrikeTimerState::Timer1, _) => StrikeTimerState::Timer2,
            (StrikeTimerState::Timer2, FighterType::Soldier) => {
                // Soldier can't do 5-hit combos, so reset after Timer2
                self.melee_cooldown = Self::POST_COMBO_3_COOLDOWN;
                StrikeTimerState::Timer1
            }
            (StrikeTimerState::Timer2, _) => StrikeTimerState::Timer3,
            (StrikeTimerState::Timer3, _) => {
                self.melee_cooldown = Self::POST_COMBO_3_COOLDOWN;
                StrikeTimerState::Timer1
            }
        };
    }
}
