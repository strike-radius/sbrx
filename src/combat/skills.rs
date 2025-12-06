// File: src/combat/skills.rs

use std::collections::HashMap;

pub const FLICKER_STRIKE_RADIUS: f64 = 500.0;
pub const FLICKER_STRIKE_DAMAGE_MULTIPLIER: f64 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillType {
    FlickerStrike,
    PulseOrb,
}

#[derive(Debug, Clone)]
pub struct Skill {
    pub skill_type: SkillType,
    pub cooldown_timer: f64,
    pub cooldown_duration: f64,
}

impl Skill {
    pub fn new(skill_type: SkillType) -> Self {
        let cooldown_duration = match skill_type {
            SkillType::FlickerStrike => 3.25,
            SkillType::PulseOrb => 2.0,
        };
        Self {
            skill_type,
            cooldown_timer: 0.0, // Ready to use immediately
            cooldown_duration,
        }
    }

    pub fn is_ready(&self) -> bool {
        self.cooldown_timer <= 0.0
    }

    pub fn trigger(&mut self) {
        self.cooldown_timer = self.cooldown_duration;
    }

    pub fn update(&mut self, dt: f64) {
        if self.cooldown_timer > 0.0 {
            self.cooldown_timer -= dt;
        }
    }
}

// This will hold all skills for one entity.
#[derive(Debug, Clone)]
pub struct SkillManager {
    pub skills: HashMap<SkillType, Skill>,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn add_skill(&mut self, skill_type: SkillType) {
        self.skills.insert(skill_type, Skill::new(skill_type));
    }

    pub fn update(&mut self, dt: f64) {
        for skill in self.skills.values_mut() {
            skill.update(dt);
        }
    }

    pub fn is_skill_ready(&self, skill_type: SkillType) -> bool {
        self.skills.get(&skill_type).map_or(false, |s| s.is_ready())
    }

    pub fn trigger_skill(&mut self, skill_type: SkillType) {
        if let Some(skill) = self.skills.get_mut(&skill_type) {
            skill.trigger();
        }
    }
}
