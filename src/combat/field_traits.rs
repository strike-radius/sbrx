// File: src/combat/field_traits.rs

use crate::game_state::FighterType;
use crate::map_system::FieldId as SbrxFieldId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatAttribute {
    Level, // Interpreted as a boost to core stats (ATK, DEF, SPD)
    _Attack,
    _Defense,
    _Speed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TraitTarget {
    _Player, // The currently controlled fighter
    Fighter(FighterType),
}

pub struct FieldTrait {
    pub field_id: SbrxFieldId,
    pub attribute: StatAttribute,
    pub modifier: i32,
    pub target: TraitTarget,
    pub description: String,
}

pub struct FieldTraitManager {
    pub traits: Vec<FieldTrait>,
	pub wilderness_trait: FieldTrait,
}

impl FieldTraitManager {
    pub fn new() -> Self {
        let mut traits = Vec::new();

        // Add the testrun trait for the racetrack
        traits.push(FieldTrait {
            field_id: SbrxFieldId(0, 0),
            attribute: StatAttribute::Level,
            modifier: 1,
            target: TraitTarget::Fighter(FighterType::Racer),
            description: "FIELD TRAIT:+[1] LVL to RACER".to_string(),
        });
		
        // SOLDIER Traits
        // Rocketbay (x-2, y5)
        traits.push(FieldTrait {
            field_id: SbrxFieldId(-2, 5),
            attribute: StatAttribute::Level,
            modifier: 1,
            target: TraitTarget::Fighter(FighterType::Soldier),
            description: "FIELD TRAIT:+[1] LVL to SOLDIER".to_string(),
        });
        // Fort Silo (x-25, y25)
        traits.push(FieldTrait {
            field_id: SbrxFieldId(-25, 25),
            attribute: StatAttribute::Level,
            modifier: 1,
            target: TraitTarget::Fighter(FighterType::Soldier),
            description: "FIELD TRAIT:+[1] LVL to SOLDIER".to_string(),
        });
        
        // Note: Bunker traits would ideally be handled dynamically based on area type,
        // but if tied strictly to field coordinates, the Fort Silo trait covers the bunker entrance field.
        // If "Bunker [x]" implies distinct floors, current static definition applies to the whole field ID.		

        // raptor Trait (Stored separately for dynamic application)
        let wilderness_trait = FieldTrait {
            field_id: SbrxFieldId(9999, 9999), // Placeholder ID
            attribute: StatAttribute::Level,
            modifier: 1,
            target: TraitTarget::Fighter(FighterType::Raptor),
            description: "FIELD TRAIT:+[1] LVL to RAPTOR".to_string(),
        };

        Self { traits, wilderness_trait }
    }

    pub fn get_active_traits_for_field(&self, field_id: &SbrxFieldId) -> Vec<&FieldTrait> {
        let mut active_traits: Vec<&FieldTrait> = self.traits
            .iter()
            .filter(|t| t.field_id == *field_id)
            .collect();
 
        // raptor Wilderness Logic
        let specific_locations = [
            SbrxFieldId(0, 0),      // Racetrack
            SbrxFieldId(-2, 5),     // Rocketbay
            SbrxFieldId(-25, 25),   // Fort Silo
        ];
 
        if !specific_locations.contains(field_id) {
            active_traits.push(&self.wilderness_trait);
        }
 
        active_traits
    }
}
