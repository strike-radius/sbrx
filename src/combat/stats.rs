// File: src/combat/stats.rs

#[derive(Debug, Clone, Copy)]
pub struct DefenseStats {
    pub hp: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AttackStats {
    pub melee_damage: f64,
    pub ranged_damage: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct SpeedStats {
    pub run_speed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Stats {
    pub defense: DefenseStats,
    pub attack: AttackStats,
    pub speed: SpeedStats,
}

// --- Base values for stat calculations (level 1) ---
pub const HP_PER_DEFENSE_POINT: f64 = 100.0; // prev 40.0
pub const DAMAGE_PER_ATTACK_POINT: f64 = 12.5;
pub const SPEED_PER_SPEED_POINT: f64 = 50.0;

// --- Fighter Stats Definitions ---

// RACER: { defence: 3, attack: 1, speed: 5 }
pub const RACER_LVL1_STATS: Stats = Stats {
    defense: DefenseStats {
        hp: 3.0 * HP_PER_DEFENSE_POINT, // 120.0 HP
    },
    attack: AttackStats {
        melee_damage: 1.0 * DAMAGE_PER_ATTACK_POINT, // 12.5 Damage
        ranged_damage: 1.0 * DAMAGE_PER_ATTACK_POINT, // 12.5 Damage
    },
    speed: SpeedStats {
        run_speed: 5.0 * SPEED_PER_SPEED_POINT,
    },
};

// SOLDIER: { defence: 5, attack: 3, speed: 1 }
pub const SOLDIER_LVL1_STATS: Stats = Stats {
    defense: DefenseStats {
        hp: 5.0 * HP_PER_DEFENSE_POINT, // 200.0 HP
    },
    attack: AttackStats {
        melee_damage: 3.0 * DAMAGE_PER_ATTACK_POINT, // 37.5 Damage
        ranged_damage: 3.0 * DAMAGE_PER_ATTACK_POINT, // 37.5 Damage
    },
    speed: SpeedStats {
        run_speed: 1.0 * SPEED_PER_SPEED_POINT,
    },
};

// HUNTER: { defence: 1, attack, 5, speed: 3 }
pub const HUNTER_LVL1_STATS: Stats = Stats {
    defense: DefenseStats {
        hp: 1.0 * HP_PER_DEFENSE_POINT,
    },
    attack: AttackStats {
        melee_damage: 5.0 * DAMAGE_PER_ATTACK_POINT,
        ranged_damage: 5.0 * DAMAGE_PER_ATTACK_POINT,
    },
    speed: SpeedStats {
        run_speed: 3.0 * SPEED_PER_SPEED_POINT,
    },
};
