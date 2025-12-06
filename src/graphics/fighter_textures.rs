// File: fighter_textures.rs

use find_folder::Search;
use piston_window::*;
use std::path::PathBuf;

/// Contains all textures for a fighter type (both on foot and on bike)
pub struct FighterTextures {
    pub idle: G2dTexture,
    pub fwd: G2dTexture,
    pub backpedal: G2dTexture,
    pub block: G2dTexture,
    pub block_break: G2dTexture,
    pub ranged: G2dTexture,
    pub ranged_marker: G2dTexture,
    pub ranged_blur: G2dTexture,
    pub rush: G2dTexture,
    pub strike: Vec<G2dTexture>,
    // Bike textures
    pub bike_idle: G2dTexture,
    pub bike_accelerate: Vec<G2dTexture>,
    pub bike_slide: G2dTexture,
    pub bike_block: G2dTexture,
    pub bike_ranged: G2dTexture,
    pub bike_rush: G2dTexture,
    pub bike_strike: Vec<G2dTexture>,
    // Boost textures (Racer specific)
    pub fwd_boost: Option<G2dTexture>,
    pub backpedal_boost: Option<G2dTexture>,
    pub bike_accelerate_boost: Option<Vec<G2dTexture>>,
    pub bike_slide_boost: Option<G2dTexture>,	
}

/// Helper function to load textures for a specific fighter type
pub fn load_fighter_textures(
    window: &mut PistonWindow,
    fighter_type: &str,
    assets: PathBuf,
) -> FighterTextures {
    // Load basic textures
    let idle_path = assets.join(format!("player/{}/idle.png", fighter_type)); // Changed
    let idle_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &idle_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| panic!("Failed to load idle texture at {:?}: {}", idle_path, e));

    let fwd_path = assets.join(format!("player/{}/fwd.png", fighter_type)); // Changed
    let fwd_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &fwd_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| panic!("Failed to load fwd texture at {:?}: {}", fwd_path, e));

    let backpedal_path = assets.join(format!("player/{}/backpedal.png", fighter_type)); // Changed
    let backpedal_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &backpedal_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load backpedal texture at {:?}: {}",
            backpedal_path, e
        )
    });

    let block_path = assets.join(format!("player/{}/block.png", fighter_type)); // Changed
    let block_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &block_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| panic!("Failed to load block texture at {:?}: {}", block_path, e));

    let ranged_path = assets.join(format!("player/{}/ranged.png", fighter_type)); // Changed
    let ranged_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &ranged_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| panic!("Failed to load ranged texture at {:?}: {}", ranged_path, e));

    let (ranged_marker_filename, ranged_blur_filename) = match fighter_type {
        "soldier" => (
            "player/soldier/bullet_pen.png",
            "player/soldier/bullet_blur.png",
        ),
        _ => (
            "player/racer/ranged_racer_shield.png",
            "player/racer/racer_shield_blur.png",
        ),
    };

    let ranged_marker_path = assets.join(ranged_marker_filename);
    let ranged_marker_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &ranged_marker_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load ranged marker texture at {:?}: {}",
            ranged_marker_path, e
        )
    });

    let ranged_blur_path = assets.join(ranged_blur_filename);
    let ranged_blur_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &ranged_blur_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load ranged blur texture at {:?}: {}",
            ranged_blur_path, e
        )
    });

    let rush_path = assets.join(format!("player/{}/rush.png", fighter_type)); // Changed
    let rush_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &rush_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| panic!("Failed to load rush texture at {:?}: {}", rush_path, e));

    // Load strike animation textures
    let mut strike_textures = Vec::new();
    for i in 1..=3 {
        let strike_path = assets.join(format!("player/{}/strike{}.png", fighter_type, i)); // Changed
        let strike_texture = Texture::from_path(
            &mut window.create_texture_context(),
            &strike_path,
            Flip::None,
            &TextureSettings::new(),
        )
        .unwrap_or_else(|e| panic!("Failed to load strike texture at {:?}: {}", strike_path, e));
        strike_textures.push(strike_texture);
    }

    let block_break_path = assets.join(format!("player/{}/block_break.png", fighter_type));
    let block_break_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &block_break_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load block_break texture at {:?}: {}",
            block_break_path, e
        )
    });

    // Load bike textures
    // Assuming 'racer_onBike' is a specific asset sub-folder name that might exist for both 'racer' and 'soldier' types.
    let bike_idle_path = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeIdle.png",
        fighter_type
    )); // Changed
    let bike_idle_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_idle_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_idle texture at {:?}: {}",
            bike_idle_path, e
        )
    });

    let mut bike_accelerate_textures = Vec::new();
    let bike_accelerate_path1 = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeAccelerate.png",
        fighter_type
    ));
    let bike_accelerate_texture1 = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_accelerate_path1,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_accelerate texture at {:?}: {}",
            bike_accelerate_path1, e
        )
    });
    bike_accelerate_textures.push(bike_accelerate_texture1);

    let bike_accelerate_path2 = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeAccelerate2.png",
        fighter_type
    ));
    let bike_accelerate_texture2 = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_accelerate_path2,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_accelerate2 texture at {:?}: {}",
            bike_accelerate_path2, e
        )
    });
    bike_accelerate_textures.push(bike_accelerate_texture2);

    let bike_slide_path = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeSlide.png",
        fighter_type
    )); // Changed
    let bike_slide_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_slide_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_slide texture at {:?}: {}",
            bike_slide_path, e
        )
    });

    let bike_block_path = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeBlock.png",
        fighter_type
    )); // Changed
    let bike_block_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_block_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_block texture at {:?}: {}",
            bike_block_path, e
        )
    });

    let bike_ranged_path = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeRanged.png",
        fighter_type
    )); // Changed
    let bike_ranged_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_ranged_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_ranged texture at {:?}: {}",
            bike_ranged_path, e
        )
    });

    let bike_rush_path = assets.join(format!(
        "player/{}/racer_onBike/rcrBikeRush.png",
        fighter_type
    )); // Changed
    let bike_rush_texture = Texture::from_path(
        &mut window.create_texture_context(),
        &bike_rush_path,
        Flip::None,
        &TextureSettings::new(),
    )
    .unwrap_or_else(|e| {
        panic!(
            "Failed to load bike_rush texture at {:?}: {}",
            bike_rush_path, e
        )
    });

    // Load bike strike textures
    let mut bike_strike_textures = Vec::new();
    for i in 1..=3 {
        let bike_strike_path = assets.join(format!(
            "player/{}/racer_onBike/rcrBikeStrike{}.png",
            fighter_type, i
        )); // Changed
        let bike_strike_texture = Texture::from_path(
            &mut window.create_texture_context(),
            &bike_strike_path,
            Flip::None,
            &TextureSettings::new(),
        )
        .unwrap_or_else(|e| {
            panic!(
                "Failed to load bike_strike texture at {:?}: {}",
                bike_strike_path, e
            )
        });
        bike_strike_textures.push(bike_strike_texture);
    }
	
    // Load Boost Textures (Only for Racer)
    let (fwd_boost, backpedal_boost, bike_accelerate_boost, bike_slide_boost) = if fighter_type == "racer" {
        let fb_path = assets.join("player/racer/fwd_boost.png");
        let fwd_b = Texture::from_path(&mut window.create_texture_context(), &fb_path, Flip::None, &TextureSettings::new()).ok();
 
        let bb_path = assets.join("player/racer/backpedal_boost.png");
        let back_b = Texture::from_path(&mut window.create_texture_context(), &bb_path, Flip::None, &TextureSettings::new()).ok();
 
        let mut bike_acc_b_vec = Vec::new();
        let bab1_path = assets.join("player/racer/racer_onBike/rcrBikeAccelerate_boost.png");
        if let Ok(tex) = Texture::from_path(&mut window.create_texture_context(), &bab1_path, Flip::None, &TextureSettings::new()) {
            bike_acc_b_vec.push(tex);
        }
        let bab2_path = assets.join("player/racer/racer_onBike/rcrBikeAccelerate2_boost.png");
        if let Ok(tex) = Texture::from_path(&mut window.create_texture_context(), &bab2_path, Flip::None, &TextureSettings::new()) {
            bike_acc_b_vec.push(tex);
        }
        let bike_acc_b = if !bike_acc_b_vec.is_empty() { Some(bike_acc_b_vec) } else { None };
 
        let bsb_path = assets.join("player/racer/racer_onBike/rcrBikeSlide_boost.png");
        let slide_b = Texture::from_path(&mut window.create_texture_context(), &bsb_path, Flip::None, &TextureSettings::new()).ok();
 
        (fwd_b, back_b, bike_acc_b, slide_b)
    } else {
        (None, None, None, None)
    };	

    FighterTextures {
        idle: idle_texture,
        fwd: fwd_texture,
        backpedal: backpedal_texture,
        block: block_texture,
        block_break: block_break_texture,
        ranged: ranged_texture,
        ranged_marker: ranged_marker_texture,
        ranged_blur: ranged_blur_texture,
        rush: rush_texture,
        strike: strike_textures,
        bike_idle: bike_idle_texture,
        bike_accelerate: bike_accelerate_textures,
        bike_slide: bike_slide_texture,
        bike_block: bike_block_texture,
        bike_ranged: bike_ranged_texture,
        bike_rush: bike_rush_texture,
        bike_strike: bike_strike_textures,
        fwd_boost,
        backpedal_boost,
        bike_accelerate_boost,
        bike_slide_boost,		
    }
}

/// Helper function to update current texture references based on fighter state and type
pub fn update_current_textures<'a>(
    fighter: &crate::entities::fighter::Fighter,
    textures: &'a FighterTextures,
    current_idle: &mut &'a G2dTexture,
    current_fwd: &mut &'a G2dTexture,
    current_backpedal: &mut &'a G2dTexture,
    current_block: &mut &'a G2dTexture,
    current_block_break: &mut &'a G2dTexture,
    current_ranged: &mut &'a G2dTexture,
    current_ranged_marker: &mut &'a G2dTexture,
    current_ranged_blur: &mut &'a G2dTexture,
    current_rush: &mut &'a G2dTexture,
    current_strike: &mut &'a Vec<G2dTexture>,
	shift_held: bool,
) {
    let use_boost = fighter.fighter_type == crate::game_state::FighterType::Racer && fighter.boost && shift_held;	
	
    match fighter.state {
        crate::game_state::RacerState::OnFoot => {
            *current_idle = &textures.idle;
            *current_fwd = if use_boost { textures.fwd_boost.as_ref().unwrap_or(&textures.fwd) } else { &textures.fwd };
            *current_backpedal = if use_boost { textures.backpedal_boost.as_ref().unwrap_or(&textures.backpedal) } else { &textures.backpedal };
            *current_block = &textures.block;
            *current_block_break = &textures.block_break;
            *current_ranged = &textures.ranged;
            *current_ranged_marker = &textures.ranged_marker;
            *current_ranged_blur = &textures.ranged_blur;
            *current_rush = &textures.rush;
            *current_strike = &textures.strike;
        }
        crate::game_state::RacerState::OnBike => {
            *current_idle = &textures.bike_idle;
            if use_boost {
                *current_fwd = textures.bike_accelerate_boost.as_ref().map(|v| &v[0]).unwrap_or(&textures.bike_accelerate[0]);
                *current_backpedal = textures.bike_slide_boost.as_ref().unwrap_or(&textures.bike_slide);
            } else {
                *current_fwd = &textures.bike_accelerate[0];
                *current_backpedal = &textures.bike_slide;
            }
            *current_block = &textures.bike_block;
            *current_block_break = &textures.block_break; // Use on-foot version for bike
            *current_ranged = &textures.bike_ranged;
            // For now, bike ranged effects are the same as on-foot for that character type
            *current_ranged_marker = &textures.ranged_marker;
            *current_ranged_blur = &textures.ranged_blur;
            *current_rush = &textures.bike_rush;
            *current_strike = &textures.bike_strike;
        }
    }
}

/// Helper function to check if a high-priority animation is active
pub fn is_high_priority_animation_active(
    rush_active: bool,
    strike_animation_timer: f64,
    block_active: bool,
    rmb_held: bool,
) -> bool {
    // Return true if any high-priority animation is active
    rush_active || strike_animation_timer > 0.0 || block_active || rmb_held
}
