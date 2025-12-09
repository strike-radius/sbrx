// file: src/entities/ground_assets.rs

use crate::map_system::FieldId as SbrxFieldId;
use rand::Rng;
use std::collections::HashSet;

// Represents a single ground asset with its properties.
pub struct GroundAsset {
    pub name: &'static str,
    pub path: &'static str,
    // A weight for randomization. Higher means more common.
    pub spawn_weight: u32,
}

// Manages the collection of ground assets and their spawning logic.
pub struct GroundAssetManager {
    assets: Vec<GroundAsset>,
    exclusion_zones: HashSet<SbrxFieldId>,
    total_spawn_weight: u32,
}

impl GroundAssetManager {
    /// Creates a new manager and populates it with all defined ground assets.
    pub fn new() -> Self {
        let assets = vec![
            GroundAsset {
                name: "broken_post",
                path: "ground/broken_post.png",
                spawn_weight: 10,
            },
            GroundAsset {
                name: "cactus",
                path: "ground/cactus.png",
                spawn_weight: 20,
            },
            GroundAsset {
                name: "cactus2",
                path: "ground/cactus2.png",
                spawn_weight: 20,
            },
            GroundAsset {
                name: "campfire_lit",
                path: "ground/campfire_lit.png",
                spawn_weight: 2,
            },
            GroundAsset {
                name: "campfire_loaded",
                path: "ground/campfire_loaded.png",
                spawn_weight: 3,
            },
            GroundAsset {
                name: "campfire_out",
                path: "ground/campfire_out.png",
                spawn_weight: 5,
            },
            GroundAsset {
                name: "cow_skull",
                path: "ground/cow_skull.png",
                spawn_weight: 8,
            },
            GroundAsset {
                name: "dead_tree",
                path: "ground/dead_tree.png",
                spawn_weight: 10,
            },
            GroundAsset {
                name: "fence",
                path: "ground/fence.png",
                spawn_weight: 5,
            },
            GroundAsset {
                name: "fence_corner",
                path: "ground/fence_corner.png",
                spawn_weight: 5,
            },
            GroundAsset {
                name: "fence_side",
                path: "ground/fence_side.png",
                spawn_weight: 5,
            },
            GroundAsset {
                name: "fence_side_double",
                path: "ground/fence_side_double.png",
                spawn_weight: 5,
            },
            GroundAsset {
                name: "log",
                path: "ground/log.png",
                spawn_weight: 15,
            },
            GroundAsset {
                name: "log_pile",
                path: "ground/log_pile.png",
                spawn_weight: 8,
            },
            GroundAsset {
                name: "plant",
                path: "ground/plant.png",
                spawn_weight: 25,
            },
            GroundAsset {
                name: "rock",
                path: "ground/rock.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "rock2",
                path: "ground/rock2.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "rock3",
                path: "ground/rock3.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "rock4",
                path: "ground/rock4.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "rock5",
                path: "ground/rock5.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "rock6",
                path: "ground/rock6.png",
                spawn_weight: 30,
            },
            GroundAsset {
                name: "tall_grass",
                path: "ground/tall_grass.png",
                spawn_weight: 25,
            },
            GroundAsset {
                name: "wagon",
                path: "ground/wagon.png",
                spawn_weight: 4,
            },
            GroundAsset {
                name: "yucca",
                path: "ground/yucca.png",
                spawn_weight: 15,
            },
            GroundAsset {
                name: "yucca2",
                path: "ground/yucca2.png",
                spawn_weight: 15,
            },
        ];

        let total_spawn_weight = assets.iter().map(|a| a.spawn_weight).sum();

        let mut exclusion_zones = HashSet::new();
        exclusion_zones.insert(SbrxFieldId(0, 0));
        exclusion_zones.insert(SbrxFieldId(-2, 5));

        Self {
            assets,
            exclusion_zones,
            total_spawn_weight,
        }
    }

    /// Checks if a given field is an exclusion zone for ground assets.
    pub fn is_exclusion_zone(&self, field_id: &SbrxFieldId) -> bool {
        self.exclusion_zones.contains(field_id)
    }

    /// Selects a random ground asset based on spawn weights.
    /// Returns `None` if there are no assets.
    pub fn get_random_asset<'a>(&'a self) -> Option<&'a GroundAsset> {
        if self.assets.is_empty() || self.total_spawn_weight == 0 {
            return None;
        }

        let mut rng = rand::rng();
        let mut choice = rng.gen_range(0..self.total_spawn_weight);

        for asset in &self.assets {
            if choice < asset.spawn_weight {
                return Some(asset);
            }
            choice -= asset.spawn_weight;
        }

        // This part should ideally not be reached if total_spawn_weight is calculated correctly.
        // It can serve as a fallback.
        self.assets.last()
    }
}
