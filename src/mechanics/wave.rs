// mechanics//wave.rs

use crate::entities::cpu_entity::CpuEntity;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveState {
    Inactive,
    Spawning, // Survival timer is active, enemies are spawning based on target count
    Frenzy,  // Timer ran out, remaining enemies get buffed
    Clearing, // Player defeated all targets before timer ran out
}

pub struct WaveStatModifiers {
    pub hp_multiplier: f64,
    pub damage_multiplier: f64,
    pub speed_multiplier: f64,
}

impl Default for WaveStatModifiers {
    fn default() -> Self {
        WaveStatModifiers {
            hp_multiplier: 1.0,
            damage_multiplier: 1.0,
            speed_multiplier: 1.0,
        }
    }
}

pub struct WaveManager {
    pub state: WaveState,
    pub current_wave: u32,
    pub total_waves: u32,
    pub survival_timer: f64,
    spawn_timer: f64,
    spawn_interval: f64,
    base_survival_duration: f64,
    // New fields for target-based mechanics
    total_targets_for_wave: u32,
    enemies_spawned_this_wave: u32,
    targets_defeated: u32,
    pub enrage_buff_applied: bool,
}

impl WaveManager {
    pub fn new() -> Self {
        Self {
            state: WaveState::Inactive,
            current_wave: 0,
            total_waves: 0,
            survival_timer: 0.0,
            spawn_timer: 0.0,
            spawn_interval: 2.5,          // Spawn an enemy every 2.5 seconds
            base_survival_duration: 30.0, // Default survival time
            total_targets_for_wave: 0,
            enemies_spawned_this_wave: 0,
            targets_defeated: 0,
            enrage_buff_applied: false,
        }
    }

    fn get_targets_for_wave(wave_num: u32) -> u32 {
        match wave_num {
            1 => 3,
            2 => 3,
            3 => 3,
            _ => 25,
        }
    }
	
    /// Returns a list of (variant, weight) for enemy spawns for the given floor.
    /// Weights are used for random selection (higher = more likely).
    pub fn get_spawn_table_for_floor(floor: i32) -> Vec<(crate::entities::cpu_entity::CpuVariant, u32)> {
        use crate::entities::cpu_entity::CpuVariant::*;
        
        match floor {
            1 => vec![(NightReaver, 1)], // Night Reavers only
            0 => vec![(LightReaver, 1)], // Light Reavers only
            -1 => vec![(LightReaver, 1), (NightReaver, 1)], // Mix
            -2 => vec![(LightReaver, 2), (NightReaver, 2), (VoidTempest, 1)], // Mix + VoidTempest
            -3 => vec![(RazorFiend, 1)], // Razor Fiend (Boss)
            _ => vec![(NightReaver, 1)], // Default fallback
        }
    }
 
    /// Helper to pick a variant from a weighted table.
    pub fn pick_random_variant(table: &Vec<(crate::entities::cpu_entity::CpuVariant, u32)>) -> crate::entities::cpu_entity::CpuVariant {
        use rand::Rng;
	    // Return default if table is empty
 	    if table.is_empty() {
 	        return crate::entities::cpu_entity::CpuVariant::NightReaver;
 	    }		
		
        let total_weight: u32 = table.iter().map(|(_, w)| w).sum();
        if total_weight == 0 { return crate::entities::cpu_entity::CpuVariant::NightReaver; } // Safe fallback
        
        let mut rng = rand::rng();
        let mut choice = rng.random_range(0..total_weight);
        
        for (variant, weight) in table {
            if choice < *weight {
                return *variant;
            }
            choice -= weight;
        }
	    // Safe: we checked table.is_empty() above
 	    table.last().map_or(crate::entities::cpu_entity::CpuVariant::NightReaver, |t| t.0)
    }	

    pub fn start_encounter(&mut self, total_waves: u32) {
        if self.is_active() {
            return;
        }
        println!(
            "[WAVE SYSTEM] Starting encounter with {} waves.",
            total_waves
        );
        self.total_waves = total_waves;
        self.start_next_wave();
    }

    fn start_next_wave(&mut self) {
        self.current_wave += 1;
        self.state = WaveState::Spawning;
        self.survival_timer = self.base_survival_duration;
        self.spawn_timer = 0.0; // Spawn first enemy immediately
        self.total_targets_for_wave = Self::get_targets_for_wave(self.current_wave);
        self.enemies_spawned_this_wave = 0;
        self.targets_defeated = 0;
        self.enrage_buff_applied = false;
        println!(
            "[WAVE SYSTEM] Starting Wave {} / {} with {} targets.",
            self.current_wave, self.total_waves, self.total_targets_for_wave
        );
    }

    /// Returns true if an enemy should be spawned this frame.
    pub fn update(&mut self, dt: f64) -> bool {
        if self.state == WaveState::Spawning {
            self.survival_timer -= dt;
        }

        // Check for transition to Frenzy state
        if self.survival_timer <= 0.0 {
            if self.state == WaveState::Spawning {
                // Only transition once
                self.state = WaveState::Frenzy;
                //println!("[WAVE SYSTEM] Survival time over. Transitioning to Frenzy state.");
            }
        }

        // Handle spawning logic
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 && self.enemies_spawned_this_wave < self.total_targets_for_wave {
            self.spawn_timer = self.spawn_interval;
            return true; // Signal to spawn an enemy
        }

        false
    }

    pub fn notify_enemy_spawned(&mut self) {
        self.enemies_spawned_this_wave += 1;
    }

    pub fn notify_enemy_defeated(&mut self) {
        if !self.is_active() {
            return;
        }

        self.targets_defeated += 1;

        // Check for wave clear condition
        if self.targets_defeated >= self.total_targets_for_wave {
            println!(
                "[WAVE SYSTEM] All targets for wave {} defeated.",
                self.current_wave
            );
            if self.current_wave < self.total_waves {
                self.start_next_wave();
            } else {
                //println!("[WAVE SYSTEM] All waves cleared. Encounter finished.");
                self.reset();
            }
        }
    }

    pub fn get_stat_modifiers_for_current_wave(&self) -> WaveStatModifiers {
        match self.current_wave {
            1 => WaveStatModifiers::default(),
            2 => WaveStatModifiers {
                hp_multiplier: 1.5,
                ..Default::default()
            },
            3 => WaveStatModifiers {
                hp_multiplier: 2.0,
                damage_multiplier: 1.25,
                ..Default::default()
            },
            4.. => WaveStatModifiers {
                hp_multiplier: 2.5,
                damage_multiplier: 1.5,
                speed_multiplier: 1.2,
            },
            _ => WaveStatModifiers::default(),
        }
    }

    pub fn apply_modifiers_to_new_cpu(&self, cpu: &mut CpuEntity) {
        let modifiers = self.get_stat_modifiers_for_current_wave();
        cpu.max_hp *= modifiers.hp_multiplier;
        cpu.current_hp = cpu.max_hp;
        cpu.damage_value *= modifiers.damage_multiplier;
        cpu.speed *= modifiers.speed_multiplier;
        // If spawning during enrage, apply the enrage buff immediately
        if self.state == WaveState::Frenzy {
            //println!("[WAVE SYSTEM] Spawning a FRENZIED enemy.");
            cpu.damage_value *= 2.0;
            cpu.speed *= 2.0;
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn is_active(&self) -> bool {
        self.state != WaveState::Inactive
    }

    pub fn get_ui_text(&self) -> Option<String> {
        if !self.is_active() {
            return None;
        }

        let targets_remaining = self
            .total_targets_for_wave
            .saturating_sub(self.targets_defeated);

        match self.state {
            WaveState::Spawning => Some(format!(
                "WAVE {} / {}   |   SURVIVE: {:.0}s   |   TARGETS: {}",
                self.current_wave, self.total_waves, self.survival_timer, targets_remaining
            )),
            WaveState::Frenzy => Some(format!(
                "WAVE {} / {}   |   FRENZY   |   TARGETS: {}",
                self.current_wave, self.total_waves, targets_remaining
            )),
            WaveState::Clearing => None, // This state is now effectively instant, no need for UI
            WaveState::Inactive => None,
        }
    }
}
