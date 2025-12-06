// Crate: firmament_lib/lib.rs - FIXED FOR RODIO 0.21.1 OGG SUPPORT (v2)

#![allow(deprecated)]

extern crate piston_window;
extern crate rand;
extern crate rodio;
extern crate image;
extern crate find_folder;
extern crate gl;

use rodio::mixer::Mixer;

// --- Re-exports for the library's public API ---
// Consumers of this library will likely need these types.
pub use piston_window::{
    PistonWindow, Key, Context, G2d, Glyphs, // Core types for interacting with the Game struct
    WindowSettings, OpenGL, EventSettings, Events, Button, // For setting up the window and event loop
    Input, Motion, // For event handling details
    RenderArgs, UpdateArgs, // For the event loop
    Transformed, // Often used with rendering
    // Common drawing functions, if the user wants to draw on top of the game
    clear, ellipse, rectangle, polygon, image as piston_image, text as piston_text
};
// Note: `map_system` types are re-exported after its definition.

// --- Original `use` statements for the library's internal implementation ---
// These bring types into scope for the code within this library.
use piston_window::*; // Kept for broader compatibility with original code structure
use rand::Rng; // The Rng trait
use rand::SeedableRng;
use rodio::{OutputStream, Sink, source::Buffered, Decoder, Source};
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::{HashMap, VecDeque};

// --- Map System Module ---
// This module is made public, and its relevant types are also public.
pub mod map_system {
    #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
    pub struct FieldId3D(pub i32, pub i32, pub i32); // x, y, z

    pub struct MapSystem {
        pub current_plane_name: String,
        pub current_field_id: FieldId3D,
    }

    impl MapSystem {
        pub fn new(plane_name: String, start_field_id: FieldId3D) -> Self {
            MapSystem {
                current_plane_name: plane_name,
                current_field_id: start_field_id,
            }
        }

        pub fn transition_field_by_delta(&mut self, dx_fields: i32, dy_fields: i32, dz_fields: i32) {
            self.current_field_id.0 += dx_fields;
            self.current_field_id.1 += dy_fields;
            self.current_field_id.2 += dz_fields;
            // Optional: Log the transition for debugging
            // println!(
            //     "Transitioned to plane '{}', field ID: x[{}], y[{}], z[{}]",
            //     self.current_plane_name, self.current_field_id.0, self.current_field_id.1, self.current_field_id.2
            // );
        }

        pub fn get_display_string(&self) -> String {
            format!(
                "{}_field.x[{}]y[{}]z[{}]",
                self.current_plane_name, self.current_field_id.0, self.current_field_id.1, self.current_field_id.2
            )
        }
    }
}
// Re-exporting FieldId3D and MapSystem for easier access from the crate root.
pub use map_system::{FieldId3D, MapSystem as GameMapSystem}; // Aliased to avoid potential name clashes if user defines their own MapSystem.

mod flying_saucer;
use flying_saucer::{FlyingSaucer, SaucerProjectile};

// --- Constants ---
// Public constants that might be useful for the library consumer
pub const WINDOW_WIDTH: f64 = 1920.0;
pub const WINDOW_HEIGHT: f64 = 1080.0;

// Internal constants (not prefixed with `pub`)
const SHIP_SIZE: f64 = 100.0;
const SHIP_ACCELERATION: f64 = 1250.0; // 750.0
const SHIP_ROTATION_SPEED: f64 = 4.5; // 
const SHIP_DRAG: f64 = 0.985;
const SHIP_MAX_SPEED: f64 = 1250.0; // 1000.0
const SHIP_INVINCIBILITY_DURATION: f64 = 2.5;
const SHIP_RUSH_SPEED: f64 = 2500.0;
const SHIP_RUSH_DURATION: f64 = 0.25;
const SHIP_RUSH_COOLDOWN: f64 = 1.0;

const BULLET_SPEED: f64 = 2500.0;
const BULLET_LIFETIME: f64 = 1.2;
const BULLET_COOLDOWN: f64 = 0.22;
const BULLET_RADIUS: f64 = 3.0;

const ASTEROID_BASE_SPEED: f64 = 600.0;
const ASTEROID_MAX_SPIN: f64 = 2.5;
const ASTEROID_POINTS_LARGE: u32 = 20;
const ASTEROID_POINTS_MEDIUM: u32 = 50;
const ASTEROID_POINTS_SMALL: u32 = 100;
const INITIAL_ASTEROIDS: usize = 0; // 5
const MAX_ASTEROIDS: usize = 20;

const UFO_SPAWN_INTERVAL_MIN: f64 = 0.1;
const UFO_SPAWN_INTERVAL_MAX: f64 = 1.0; // test
const UFO_SPEED: f64 = 250.0;
const UFO_SHOOT_COOLDOWN: f64 = 2.0;
const UFO_POINTS: u32 = 200;
const UFO_RADIUS: f64 = 25.0;
const ENEMY_BULLET_SPEED: f64 = 250.0;
const ENEMY_BULLET_LIFETIME: f64 = 2.0;
const ENEMY_BULLET_RADIUS: f64 = 4.0;

const PARTICLE_LIFETIME_MIN: f64 = 0.3;
const PARTICLE_LIFETIME_MAX: f64 = 0.8;
const PARTICLE_SPEED_MIN: f64 = 30.0;
const PARTICLE_SPEED_MAX: f64 = 100.0;
const PARTICLE_COUNT_ASTEROID: usize = 15;
const PARTICLE_COUNT_SHIP: usize = 30;
const PARTICLE_COUNT_UFO: usize = 25;

const WARNING_MESSAGE_DURATION: f64 = 3.0;

const STARTING_SHIELDS: u32 = 10; 
//const STAR_COUNT: usize = 0; // 300 

// --- Helper Functions (internal to the library) ---
fn wrap_position(pos: &mut [f64; 2], size: f64) {
    let half_size = size / 2.0;
    if pos[0] < -half_size { pos[0] = WINDOW_WIDTH + half_size; }
    if pos[0] > WINDOW_WIDTH + half_size { pos[0] = -half_size; }
    if pos[1] < -half_size { pos[1] = WINDOW_HEIGHT + half_size; }
    if pos[1] > WINDOW_HEIGHT + half_size { pos[1] = -half_size; }
}

fn distance_sq(p1: [f64; 2], p2: [f64; 2]) -> f64 {
    (p1[0] - p2[0]).powi(2) + (p1[1] - p2[1]).powi(2)
}

fn normalize_vector(vec: &mut [f64; 2]) {
    let mag = (vec[0].powi(2) + vec[1].powi(2)).sqrt();
    if mag > 0.0 {
        vec[0] /= mag;
        vec[1] /= mag;
    }
}

fn debug_print(_msg: &str) {
    // To enable debug prints, uncomment the next line
    println!("[DEBUG] {}", _msg);
}

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum FirmamentDeathCause {
    Standard,
    FlyingSaucer,
}

// --- Enums and Structs (internal to the library) ---
#[derive(Copy, Clone, PartialEq, Debug)]
enum AsteroidSize {
    Large,
    Medium,
    Small,
}

impl AsteroidSize {
    fn radius(&self) -> f64 {
        match self {
            AsteroidSize::Large => 35.0,
            AsteroidSize::Medium => 20.0,
            AsteroidSize::Small => 10.0,
        }
    }
    fn points(&self) -> u32 {
        match self {
            AsteroidSize::Large => ASTEROID_POINTS_LARGE,
            AsteroidSize::Medium => ASTEROID_POINTS_MEDIUM,
            AsteroidSize::Small => ASTEROID_POINTS_SMALL,
        }
    }
}

struct GameObject {
    pos: [f64; 2],
    vel: [f64; 2],
    rot: f64,
    radius: f64,
    active: bool,
}

struct Player {
    obj: GameObject,
    is_thrusting: bool,
	is_braking: bool,
    rotating_left: bool,
    rotating_right: bool,
    shields: u32,
    score: u32,
    shoot_cooldown: f64,
    invincible_timer: f64,
    rush_active: bool,
    rush_timer: f64,
    rush_cooldown: f64,
}

struct Asteroid {
    obj: GameObject,
    size: AsteroidSize,
    spin: f64,
}

struct Bullet {
    obj: GameObject,
    lifetime: f64,
}

struct EnemyBullet {
    obj: GameObject,
    lifetime: f64,
}

struct Ufo {
    obj: GameObject,
    shoot_timer: f64,
    _target_player_pos: Option<[f64; 2]>,
    change_dir_timer: f64,
}

struct Particle {
    pos: [f64; 2],
    vel: [f64; 2],
    lifetime: f64,
    max_lifetime: f64,
    color: [f32; 4],
    size: f64,
}

struct SpeedLine {
    start_pos: [f64; 2],
    end_pos: [f64; 2],
    lifetime: f64,
    max_lifetime: f64,
    color: [f32; 4],
    width: f64,
}
/*
struct Star {
    pos: [f64; 2],
    speed: f64,
    size: f64,
    color: [f32; 4],
}
*/
struct Assets {
    textures: HashMap<String, G2dTexture>,
    sounds: HashMap<String, Buffered<Decoder<std::io::BufReader<std::fs::File>>>>,
    _assets_path: PathBuf,
	ground_textures_air: Vec<G2dTexture>, // step 1
}

impl Assets {
    // Internal method to load assets.
    fn load(window: &mut PistonWindow) -> Result<Assets, String> {
        debug_print("Attempting to load assets...");

        // Prioritize firmament_lib specific path to avoid conflict with main game assets
        let mut search_paths = Vec::new();
		//search_paths.push(std::path::PathBuf::from("crates/firmament_lib/assets"));
        search_paths.push(std::path::PathBuf::from("assets/firmament_lib"));
        search_paths.push(std::path::PathBuf::from("firmament_lib")); // Fallback/Release
 
        let assets_path = search_paths.into_iter()
            .find(|p| p.exists())
            .or_else(|| {
                find_folder::Search::ParentsThenKids(3, 3)
                    .for_folder("firmament_lib")
                    .ok()
            })
            .ok_or_else(|| {
                let err_msg = "Failed to find 'assets' folder".to_string();
                debug_print(&err_msg);
                err_msg
            })?;

        debug_print(&format!("Assets path found: {:?}", assets_path));

        let mut textures = HashMap::new();
        let mut sounds: HashMap<String, Buffered<Decoder<std::io::BufReader<std::fs::File>>>> = HashMap::new();

        // PistonWindow types like G2dTexture, Texture, TextureSettings, Filter are used here.
        // image crate is used for image loading.
        let texture_context = &mut window.create_texture_context();
        let mut load_texture = |name: &str| -> Result<(), String> {
            let path = assets_path.join("images").join(format!("{}.png", name));
            debug_print(&format!("Attempting to load texture: {:?}", path));

            let img_buffer = match image::open(&path) {
                Ok(img) => {
                    debug_print(&format!("Successfully loaded image: {}", name));
                    img.to_rgba8()
                },
                Err(e) => {
                    let err_msg = format!("Failed to load image {}: {}", path.display(), e);
                    debug_print(&err_msg);
                    return Err(err_msg);
                }
            };

            let texture = match Texture::from_image( // piston_window::Texture
                texture_context,
                &img_buffer,
                &TextureSettings::new().filter(Filter::Nearest), // piston_window::TextureSettings, piston_window::Filter
            ) {
                Ok(tex) => {
                    debug_print(&format!("Successfully created texture: {}", name));
                    tex
                },
                Err(e) => {
                    let err_msg = format!("Failed to create texture {}: {}", path.display(), e);
                    debug_print(&err_msg);
                    return Err(err_msg);
                }
            };

            textures.insert(name.to_string(), texture);
            Ok(())
        };

        if let Err(e) = load_texture("fighter_jet") { debug_print(&format!("Warning: {}", e)); }
        if let Err(e) = load_texture("asteroid") { debug_print(&format!("Warning: {}", e)); }
        if let Err(e) = load_texture("bullet") { debug_print(&format!("Warning: {}", e)); }
        if let Err(e) = load_texture("ufo") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("rocketbay_air") { debug_print(&format!("Warning: {}", e)); }
        if let Err(e) = load_texture("racetrack0_air") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("fort_silo_air") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("flying_saucer") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("plasma_blast") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("pulse_orb") { debug_print(&format!("Warning: {}", e)); }
		if let Err(e) = load_texture("inputs") { debug_print(&format!("Warning: {}", e)); }
		
		let mut ground_textures_air = Vec::new();
		let ground_air_path = assets_path.join("images").join("ground_texture_air.png");
		let flips = [Flip::None, Flip::Horizontal, Flip::Vertical, Flip::Both];
	
		for flip in &flips {
           match Texture::from_path(
               texture_context,
               &ground_air_path,
               *flip,
               &TextureSettings::new(),
           ) {
               Ok(texture) => ground_textures_air.push(texture),
               Err(e) => {
                   debug_print(&format!("Warning: Failed to load ground_texture_air with flip {:?}: {}", flip, e));
               }
           }
       }
       if ground_textures_air.is_empty() {
           debug_print("Critical: No ground_texture_air textures were loaded. Tiling will fail.");
       }
		
        // rodio types Decoder, Source, Buffered are used here.
		let mut load_sound = |name: &str, extension: &str| -> Result<(), String> {
			let path = assets_path.join("sounds").join(format!("{}.{}", name, extension));
			debug_print(&format!("Attempting to load sound: {:?}", path));

			let file = match std::fs::File::open(&path) {
				Ok(file) => {
					debug_print(&format!("Successfully opened sound file: {}.{}", name, extension));
					file
				},
				Err(e) => {
					let err_msg = format!("Failed to open sound file {}: {}", path.display(), e);
					debug_print(&err_msg);
					return Err(err_msg);
				}
			};

			// FIXED: Use Decoder::try_from instead of Decoder::new with BufReader
			// This is required for proper OGG/Vorbis decoding in rodio 0.21.1
			let source = match Decoder::try_from(file) {
				Ok(source) => {
					debug_print(&format!("Successfully decoded sound: {}.{}", name, extension));
					// Log audio info for debugging
					debug_print(&format!("  Sample rate: {}, Channels: {}", 
							   source.sample_rate(), source.channels()));
					source
				},
				Err(e) => {
					let err_msg = format!("Failed to decode sound file {}: {}", path.display(), e);
					debug_print(&err_msg);
					return Err(err_msg);
				}
			};

			sounds.insert(name.to_string(), source.buffered());
			Ok(())
		};

        let extensions = ["wav", "ogg"]; // sound effects
        for name in &[
            "shoot", "thrust", "explode_asteroid", "explode_ship",
            "ufo_spawn", "ufo_shoot", "ufo_explode"
        ] {
            let mut success = false;
            for ext in &extensions {
                if load_sound(name, ext).is_ok() {
                    success = true;
                    break;
                }
            }
            if !success {
                debug_print(&format!("Warning: Could not load any version of sound '{}'", name));
            }
        }

        debug_print("Assets loading completed!");
        Ok(Assets { textures, sounds, _assets_path: assets_path, ground_textures_air })
    }

    fn get_texture(&self, name: &str) -> Option<&G2dTexture> {
        let tex = self.textures.get(name);
        if tex.is_none() {
            debug_print(&format!("Warning: Texture '{}' not found!", name));
        }
        tex
    }

	fn play_sound(&self, mixer: &Arc<Mixer>, name: &str) {
		if let Some(sound) = self.sounds.get(name) {
			let sink = Sink::connect_new(mixer);
			sink.set_volume(1.0);  // Ensure volume is max
			debug_print(&format!("Playing sound: {}", name));
			sink.append(sound.clone());
			
			// Debug info
			debug_print(&format!("  Sink empty: {}, paused: {}", sink.empty(), sink.is_paused()));
			
			sink.detach(); // Play and forget
		} else {
			debug_print(&format!("Warning: Sound '{}' not found.", name));
		}
	}
}

// --- Game Struct (Public API) ---
pub struct Game {
    // Fields are kept private to encapsulate game logic.
    // Access to game state is provided via public getter methods.
    player: Player,
    asteroids: Vec<Asteroid>,
    bullets: Vec<Bullet>,
    ufos: Vec<Ufo>,
    enemy_bullets: Vec<EnemyBullet>,
    particles: Vec<Particle>,
	speed_lines: Vec<SpeedLine>,
//    stars: Vec<Star>,
    window_size: [f64; 2],
    rng: rand::rngs::ThreadRng, // Specific RNG type from `rand` crate
    game_over: bool,
    waiting_to_start: bool,
    assets: Assets,
    _stream: OutputStream,
    mixer: Arc<Mixer>,
    thrust_sink: Sink, // rodio::Sink
    ufo_spawn_timer: f64,
    frame_count: u64,
    pub is_paused: bool,
    map_system: map_system::MapSystem, // Using the qualified type name
    field_colors: HashMap<map_system::FieldId3D, [f32; 4]>,
	field_ground_texture_indices: HashMap<map_system::FieldId3D, usize>, // step 2
    flying_saucer: Option<FlyingSaucer>,
    saucer_projectiles: Vec<SaucerProjectile>,
    boss_fight_active: bool,
	fort_silo_landed: bool,
	pub death_cause: Option<FirmamentDeathCause>,
	warnings: VecDeque<(String, f64)>,
	pub task_bar_open: bool,
}

impl Game {
    /// Creates a new game instance.
    /// Requires a mutable reference to a PistonWindow for asset loading and context.
    pub fn new(window: &mut PistonWindow, start_field_id_override: Option<FieldId3D>, boss_already_defeated: bool, fort_silo_already_landed: bool) -> Result<Self, String> {
        debug_print("Initializing game library...");
        let assets = Assets::load(window)?;
//
		let stream = rodio::OutputStreamBuilder::open_default_stream()
			.map_err(|e| format!("Failed to open audio stream: {}", e))?;

		let mixer = stream.mixer().clone();

		let thrust_sink = Sink::connect_new(&mixer);

		if let Some(thrust_sound) = assets.sounds.get("thrust") {
			thrust_sink.append(thrust_sound.clone().repeat_infinite());
			thrust_sink.pause();
		} else {
			debug_print("Warning: Thrust sound not loaded, thrust audio will not play.");
		}
//

        // Use the override if provided, otherwise default
        let initial_field_id = start_field_id_override.unwrap_or_else(|| {
            debug_print("No start_field_id_override provided for FIRMAMENT, defaulting to 0,0,0.");
            FieldId3D(0, 0, 0)
        });
        let map_system = map_system::MapSystem::new("FIRMAMENT".to_string(), initial_field_id);
        debug_print(&format!("FIRMAMENT MapSystem initialized to: {:?}", initial_field_id));


        let mut game = Game {
            player: Player {
                obj: GameObject {
                    pos: [WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0],
                    vel: [0.0, 0.0],
                    rot: -std::f64::consts::FRAC_PI_2,
                    radius: SHIP_SIZE / 2.0,
                    active: true,
                },
                is_thrusting: false,
				is_braking: false,
                rotating_left: false,
                rotating_right: false,
                shields: STARTING_SHIELDS,
                score: 0,
                shoot_cooldown: 0.0,
                invincible_timer: SHIP_INVINCIBILITY_DURATION,
                rush_active: false,
                rush_timer: 0.0,
                rush_cooldown: 0.0,				
            },
            asteroids: Vec::new(),
            bullets: Vec::new(),
            ufos: Vec::new(),
            enemy_bullets: Vec::new(),
            particles: Vec::new(),
			speed_lines: Vec::new(),
//            stars: Vec::new(),
            window_size: [WINDOW_WIDTH, WINDOW_HEIGHT],
            rng: rand::thread_rng(),
            game_over: false,
            waiting_to_start: false,
            assets,
            _stream: stream,
            mixer: mixer.into(),
            thrust_sink,
            ufo_spawn_timer: UFO_SPAWN_INTERVAL_MAX,
            frame_count: 0,
            is_paused: false,
            map_system, // Use the initialized map_system
            field_colors: HashMap::new(),
			field_ground_texture_indices: HashMap::new(), // step 3
            flying_saucer: None,
            saucer_projectiles: Vec::new(),
            boss_fight_active: boss_already_defeated,
			fort_silo_landed: fort_silo_already_landed,
			death_cause: None,
			warnings: VecDeque::new(),
			task_bar_open: false,
        };

        // game.add_stars(STAR_COUNT);
        game.add_asteroids(INITIAL_ASTEROIDS);
        game.reset_player_position(); // This might reset the field if not careful, ensure reset_player_position doesn't touch map_system.current_field_id
        debug_print("Game library initialization complete!");
        Ok(game)
    }

    fn get_current_field_color(&mut self) -> [f32; 4] {
       let field_id = self.map_system.current_field_id;

       // Check cache first
       if let Some(&color) = self.field_colors.get(&field_id) {
           return color;
       }

       // Generate and cache new color
       // Use a simple hashing function for the seed to ensure consistent colors per field
       let p1: u64 = 73856093;
       let p2: u64 = 19349663;
       let p3: u64 = 83492791;
       let seed = (field_id.0 as u64).wrapping_mul(p1)
                ^ (field_id.1 as u64).wrapping_mul(p2)
                ^ (field_id.2 as u64).wrapping_mul(p3);

       let mut field_rng = rand::rngs::StdRng::seed_from_u64(seed);

       // Same grayscale setup as FLATLINE ground
       let gray_val = field_rng.gen_range(0.25..=0.75);
       let color = [gray_val, gray_val, gray_val, 1.0];

       self.field_colors.insert(field_id, color);
       color
    }
	
	fn get_ground_texture_index(&mut self) -> usize {
	   let field_id = self.map_system.current_field_id;
	   
	   // Check if we already have an index for this field
	   if let Some(&index) = self.field_ground_texture_indices.get(&field_id) {
		   return index;
	   }
	   
	   // Generate a new index, ensuring it's different from adjacent fields if possible
	   let seed = (field_id.0 as u64).wrapping_mul(73856093)
				^ (field_id.1 as u64).wrapping_mul(19349663)
				^ (field_id.2 as u64).wrapping_mul(83492791);
	   let mut field_rng = rand::rngs::StdRng::seed_from_u64(seed);
	   
	   let available_textures = self.assets.ground_textures_air.len();
	   if available_textures == 0 {
		   return 0;
	   }
	   
	   // Get indices from adjacent fields to avoid duplicates
	   let adjacent_fields = [
		   FieldId3D(field_id.0 - 1, field_id.1, field_id.2),
		   FieldId3D(field_id.0 + 1, field_id.1, field_id.2),
		   FieldId3D(field_id.0, field_id.1 - 1, field_id.2),
		   FieldId3D(field_id.0, field_id.1 + 1, field_id.2),
	   ];
	   
	   let mut used_indices: std::collections::HashSet<usize> = std::collections::HashSet::new();
	   for adj_field in &adjacent_fields {
		   if let Some(&index) = self.field_ground_texture_indices.get(adj_field) {
			   used_indices.insert(index);
		   }
	   }
	   
	   // Try to find an unused index
	   let mut attempts = 0;
	   let mut chosen_index;
	   loop {
		   chosen_index = field_rng.gen_range(0..available_textures);
		   if !used_indices.contains(&chosen_index) || attempts > 10 {
			   break;
		   }
		   attempts += 1;
	   }
	   
	   self.field_ground_texture_indices.insert(field_id, chosen_index);
	   chosen_index
	}

    // --- Internal Game Logic Methods ---
    // (These are not `pub` and are only callable within this module/impl block)
    fn reset_player_position(&mut self) {
        self.player.obj.pos = [WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 2.0];
        self.player.obj.vel = [0.0, 0.0];
        self.player.obj.rot = -std::f64::consts::FRAC_PI_2;
        self.player.invincible_timer = SHIP_INVINCIBILITY_DURATION;
        self.player.rush_active = false;
        self.player.rush_timer = 0.0;
		self.player.rush_cooldown = 0.0;		
    }

    fn reset_game_state(&mut self) {
        self.player.shields = STARTING_SHIELDS;
        self.player.score = 0;
        self.asteroids.clear();
        self.bullets.clear();
        self.ufos.clear();
        self.enemy_bullets.clear();
        self.particles.clear();
		self.speed_lines.clear();
        self.add_asteroids(INITIAL_ASTEROIDS);
        self.reset_player_position();
        self.game_over = false;
        self.waiting_to_start = false;
        self.ufo_spawn_timer = self.rng.gen_range(UFO_SPAWN_INTERVAL_MIN..UFO_SPAWN_INTERVAL_MAX);
        self.map_system.current_field_id = map_system::FieldId3D(-2,5,0);
    }

    fn reset_field_objects(&mut self) {
        self.asteroids.clear();
        self.ufos.clear();
        self.bullets.clear();
        self.enemy_bullets.clear();
        self.particles.clear();
		self.speed_lines.clear();
        self.add_asteroids(INITIAL_ASTEROIDS);
        self.ufo_spawn_timer = self.rng.gen_range(UFO_SPAWN_INTERVAL_MIN..UFO_SPAWN_INTERVAL_MAX);
        debug_print("Field objects reset for new field.");
    }

    fn spawn_particles(&mut self, pos: [f64; 2], count: usize, base_color: [f32; 4], speed_range: (f64, f64), lifetime_range: (f64, f64), base_size: f64) {
        for _ in 0..count {
            let angle = self.rng.gen_range(0.0..std::f64::consts::TAU);
            let speed = self.rng.gen_range(speed_range.0..speed_range.1);
            let lifetime = self.rng.gen_range(lifetime_range.0..lifetime_range.1);
            let color_variance = self.rng.gen_range(-0.2..0.2);
            let particle_color = [
                (base_color[0] + color_variance).clamp(0.0, 1.0),
                (base_color[1] + color_variance).clamp(0.0, 1.0),
                (base_color[2] + color_variance).clamp(0.0, 1.0),
                base_color[3] * self.rng.gen_range(0.7..1.0),
            ];
            let size_variance = self.rng.gen_range(0.5..1.5);

            self.particles.push(Particle {
                pos,
                vel: [angle.cos() * speed, angle.sin() * speed],
                lifetime,
                max_lifetime: lifetime,
                color: particle_color,
                size: base_size * size_variance,
            });
        }
    }
	
    fn spawn_speed_lines(&mut self) {
        let ship_pos = self.player.obj.pos;
        let ship_rot = self.player.obj.rot;
        
        // Spawn two lines slightly offset to the sides of the ship
        for side in [-1.0, 1.0].iter() {
            let offset_dist = 20.0;
            let perp_angle = ship_rot + std::f64::consts::FRAC_PI_2;
            
            let start_x = ship_pos[0] + perp_angle.cos() * offset_dist * side;
            let start_y = ship_pos[1] + perp_angle.sin() * offset_dist * side;
            
            // Lines trailing behind
            let line_len = 150.0;
            let end_x = start_x - ship_rot.cos() * line_len;
            let end_y = start_y - ship_rot.sin() * line_len;
            
            self.speed_lines.push(SpeedLine {
                start_pos: [start_x, start_y],
                end_pos: [end_x, end_y],
                lifetime: 0.3,
                max_lifetime: 0.3,
                color: [0.0, 1.0, 0.0, 0.8], // Green with alpha
                width: 2.0,
            });
        }
    }	
/*
    fn add_stars(&mut self, count: usize) {
        for _ in 0..count {
            self.stars.push(Star {
                pos: [
                    self.rng.gen_range(0.0..self.window_size[0]),
                    self.rng.gen_range(0.0..self.window_size[1]),
                ],
                speed: 0.0, // Not used.
                size: 1.0,  // "pixel" size.
                color: [1.0, 1.0, 1.0, 1.0], // "white".
            });
        }
    }
*/
    fn add_asteroids(&mut self, count: usize) {
        for _ in 0..count {
            if self.asteroids.len() >= MAX_ASTEROIDS { break; }
            self.spawn_asteroid(AsteroidSize::Large, None);
        }
    }

    fn spawn_asteroid(&mut self, size: AsteroidSize, position: Option<[f64; 2]>) {
        let pos = position.unwrap_or_else(|| {
            loop {
                let edge = self.rng.gen_range(0..4);
                let (x, y) = match edge {
                    0 => (self.rng.gen_range(0.0..self.window_size[0]), -size.radius()),
                    1 => (self.rng.gen_range(0.0..self.window_size[0]), self.window_size[1] + size.radius()),
                    2 => (-size.radius(), self.rng.gen_range(0.0..self.window_size[1])),
                    _ => (self.window_size[0] + size.radius(), self.rng.gen_range(0.0..self.window_size[1])),
                };
                let player_safe_radius: f64 = 150.0;
                 if distance_sq([x,y], self.player.obj.pos) > player_safe_radius.powi(2) {
                    break [x, y];
                }
            }
        });

        let angle = self.rng.gen_range(0.0..std::f64::consts::TAU);
        let speed_multiplier = match size {
            AsteroidSize::Large => 1.0,
            AsteroidSize::Medium => 1.2,
            AsteroidSize::Small => 1.4,
        };
        let speed = ASTEROID_BASE_SPEED * speed_multiplier * (1.0 + self.rng.gen_range(-0.2..0.2));
        let vel = [angle.cos() * speed, angle.sin() * speed];
        let spin = self.rng.gen_range(-ASTEROID_MAX_SPIN..ASTEROID_MAX_SPIN);

        self.asteroids.push(Asteroid {
            obj: GameObject { pos, vel, rot: self.rng.gen_range(0.0..std::f64::consts::TAU), radius: size.radius(), active: true },
            size, spin,
        });
    }

    fn break_asteroid(&mut self, index: usize) {
        let asteroid_pos = self.asteroids[index].obj.pos;
        let asteroid_size = self.asteroids[index].size;

        self.spawn_particles(asteroid_pos, PARTICLE_COUNT_ASTEROID, [0.6, 0.4, 0.2, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX * 0.8), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX * 0.8), 3.0);

        self.player.score += asteroid_size.points();
        self.assets.play_sound(&self.mixer, "explode_asteroid");
        self.asteroids.remove(index);

        match asteroid_size {
            AsteroidSize::Large => {
                self.spawn_asteroid(AsteroidSize::Medium, Some(asteroid_pos));
                self.spawn_asteroid(AsteroidSize::Medium, Some(asteroid_pos));
            }
            AsteroidSize::Medium => {
                self.spawn_asteroid(AsteroidSize::Small, Some(asteroid_pos));
                self.spawn_asteroid(AsteroidSize::Small, Some(asteroid_pos));
            }
            AsteroidSize::Small => {}
        }

        if self.asteroids.is_empty() && self.ufos.is_empty() && !self.game_over {
            let wave_bonus = (self.player.score / 1000) as usize;
            self.add_asteroids(INITIAL_ASTEROIDS + wave_bonus);
            self.reset_player_position();
            self.player.invincible_timer = SHIP_INVINCIBILITY_DURATION + 1.0;
            self.ufo_spawn_timer = self.rng.gen_range(UFO_SPAWN_INTERVAL_MIN..UFO_SPAWN_INTERVAL_MAX);
        }
    }

    fn shoot_player_bullet(&mut self) {
        if self.player.shoot_cooldown <= 0.0 && (!self.game_over || self.is_paused) {
            let ship_rot = self.player.obj.rot;
            let ship_pos = self.player.obj.pos;
            let spawn_offset = self.player.obj.radius + 135.0;

            let bullet_pos = [
                ship_pos[0] + ship_rot.cos() * spawn_offset,
                ship_pos[1] + ship_rot.sin() * spawn_offset,
            ];
            let bullet_vel = [
                self.player.obj.vel[0] + ship_rot.cos() * BULLET_SPEED,
                self.player.obj.vel[1] + ship_rot.sin() * BULLET_SPEED,
            ];

            self.bullets.push(Bullet {
                obj: GameObject { pos: bullet_pos, vel: bullet_vel, rot: ship_rot, radius: BULLET_RADIUS, active: true },
                lifetime: BULLET_LIFETIME,
            });
            self.player.shoot_cooldown = BULLET_COOLDOWN;
            self.assets.play_sound(&self.mixer, "shoot");
        }
    }

    fn spawn_ufo(&mut self) {
        if !self.ufos.is_empty() { return; }

        let side = self.rng.gen_range(0..2);
        let y_pos = self.rng.gen_range(UFO_RADIUS * 2.0 .. self.window_size[1] - UFO_RADIUS * 2.0);
        let (x_pos, x_vel_dir) = if side == 0 {
            (-UFO_RADIUS, 1.0)
        } else {
            (self.window_size[0] + UFO_RADIUS, -1.0)
        };

        self.ufos.push(Ufo {
            obj: GameObject {
                pos: [x_pos, y_pos],
                vel: [x_vel_dir * UFO_SPEED, 0.0],
                rot: 0.0,
                radius: UFO_RADIUS,
                active: true,
            },
            shoot_timer: self.rng.gen_range(0.5..UFO_SHOOT_COOLDOWN),
            _target_player_pos: None,
            change_dir_timer: self.rng.gen_range(3.0..6.0),
        });
        self.assets.play_sound(&self.mixer, "ufo_spawn");
        debug_print("UFO spawned");
    }

    fn update_player(&mut self, dt: f64) {
        if self.player.invincible_timer > 0.0 { self.player.invincible_timer -= dt; }
        if self.player.shoot_cooldown > 0.0 { self.player.shoot_cooldown -= dt; }	
        if self.player.rush_cooldown > 0.0 { self.player.rush_cooldown -= dt; } 
		
        if self.player.rush_active {
            self.player.rush_timer -= dt;
            if self.player.rush_timer <= 0.0 {
                self.player.rush_active = false;
                // Decelerate after rush
                self.player.obj.vel[0] *= 0.5;
                self.player.obj.vel[1] *= 0.5;
            } else {
                // Spawn speed lines continuously during rush
                if self.frame_count % 3 == 0 { // Every few frames to avoid too much clutter
                    self.spawn_speed_lines();
                }				
				
                // Rush Movement
                let rush_vec = [
                    self.player.obj.rot.cos() * SHIP_RUSH_SPEED,
                    self.player.obj.rot.sin() * SHIP_RUSH_SPEED,
                ];
                self.player.obj.vel = rush_vec;
                
                // Rush Physics Update (simplified)
                self.player.obj.pos[0] += self.player.obj.vel[0] * dt;
                self.player.obj.pos[1] += self.player.obj.vel[1] * dt;
                
                // Wrap logic for rush
                let player_radius = self.player.obj.radius;
                wrap_position(&mut self.player.obj.pos, player_radius * 2.0);
 
                // Check collisions during rush
                self.handle_rush_collisions();
                return; // Skip normal physics
            }
        }		

        if self.player.rotating_left { self.player.obj.rot -= SHIP_ROTATION_SPEED * dt; }
        if self.player.rotating_right { self.player.obj.rot += SHIP_ROTATION_SPEED * dt; }

        if self.player.is_thrusting {
            if self.thrust_sink.is_paused() { self.thrust_sink.play(); }
            if !self.is_paused { // Only apply physics if not paused
                let accel_vec = [
                    self.player.obj.rot.cos() * SHIP_ACCELERATION * dt,
                    self.player.obj.rot.sin() * SHIP_ACCELERATION * dt,
                ];
                self.player.obj.vel[0] += accel_vec[0];
                self.player.obj.vel[1] += accel_vec[1];
            }
        } else {
            if !self.thrust_sink.is_paused() { self.thrust_sink.pause(); }
        }
        
        // --- Physics and Movement (only when not paused) ---
        if self.is_paused { return; }

        let speed_sq = self.player.obj.vel[0].powi(2) + self.player.obj.vel[1].powi(2);
        if speed_sq > SHIP_MAX_SPEED.powi(2) {
            let speed = speed_sq.sqrt();
            self.player.obj.vel[0] = (self.player.obj.vel[0] / speed) * SHIP_MAX_SPEED;
            self.player.obj.vel[1] = (self.player.obj.vel[1] / speed) * SHIP_MAX_SPEED;
        }

        let current_drag = if self.player.is_braking { 0.90 } else { SHIP_DRAG }; // 0.90 is a strong braking force
        self.player.obj.vel[0] *= current_drag.powf(dt * 60.0);
        self.player.obj.vel[1] *= current_drag.powf(dt * 60.0);

        self.player.obj.pos[0] += self.player.obj.vel[0] * dt;
        self.player.obj.pos[1] += self.player.obj.vel[1] * dt;

        let pos_before_wrap_x = self.player.obj.pos[0];
        let pos_before_wrap_y = self.player.obj.pos[1];
        let player_radius = self.player.obj.radius;

        wrap_position(&mut self.player.obj.pos, player_radius * 2.0);

        let mut dx_field = 0;
        let mut dy_field = 0;
		
        // Check if boss fight is active - if so, prevent field transitions
        // Keep player locked in Fort Silo until they LAND the fighterjet
        // Unlock once TASK: LAND ON FORT SILO is complete
        if self.boss_fight_active && !self.fort_silo_landed {
            // Lock player to Fort Silo field (-25, 25)
            if self.map_system.current_field_id != map_system::FieldId3D(-25, 25, 0) {
                // Force back to Fort Silo if somehow outside
                self.map_system.current_field_id = map_system::FieldId3D(-25, 25, 0);
                self.reset_field_objects();
                debug_print("Boss fight active - forced back to Fort Silo field");
            }
            
            // Prevent any field transitions by skipping the wrap detection
            // Player can move within field but cannot leave
        } else {
            // Normal field transition logic (only when NOT in boss fight)		

			if pos_before_wrap_x < -player_radius && self.player.obj.pos[0] > self.window_size[0] {
				dx_field = -1;
			}
			else if pos_before_wrap_x > self.window_size[0] + player_radius && self.player.obj.pos[0] < 0.0 {
				dx_field = 1;
			}

			if pos_before_wrap_y < -player_radius && self.player.obj.pos[1] > self.window_size[1] {
				dy_field = 1;
			}
			else if pos_before_wrap_y > self.window_size[1] + player_radius && self.player.obj.pos[1] < 0.0 {
				dy_field = -1;
			}

			if dx_field != 0 || dy_field != 0 {
				self.map_system.transition_field_by_delta(dx_field, dy_field, 0);
				self.reset_field_objects();
                // Display gravitational force message when entering Fort Silo
                if self.map_system.current_field_id == map_system::FieldId3D(-25, 25, 0) {
                    debug_print("Player entered FIRMAMENT Fort Silo - gravitational force active");
                }				
			}
		}
    }

    fn update_asteroids(&mut self, dt: f64) {
        for asteroid in &mut self.asteroids {
            asteroid.obj.pos[0] += asteroid.obj.vel[0] * dt;
            asteroid.obj.pos[1] += asteroid.obj.vel[1] * dt;
            asteroid.obj.rot += asteroid.spin * dt;
            wrap_position(&mut asteroid.obj.pos, asteroid.obj.radius * 2.0);
        }
    }

    fn update_bullets(&mut self, dt: f64) {
        for bullet in self.bullets.iter_mut() {
            if !self.is_paused {
                bullet.obj.pos[0] += bullet.obj.vel[0] * dt;
                bullet.obj.pos[1] += bullet.obj.vel[1] * dt;
            }
            bullet.lifetime -= dt;
            if bullet.lifetime <= 0.0 { bullet.obj.active = false; }
        }
        self.bullets.retain(|b| b.obj.active &&
            b.obj.pos[0] > -b.obj.radius && b.obj.pos[0] < self.window_size[0] + b.obj.radius &&
            b.obj.pos[1] > -b.obj.radius && b.obj.pos[1] < self.window_size[1] + b.obj.radius
        );
    }

    fn update_enemy_bullets(&mut self, dt: f64) {
        for bullet in self.enemy_bullets.iter_mut() {
            if !self.is_paused {
                bullet.obj.pos[0] += bullet.obj.vel[0] * dt;
                bullet.obj.pos[1] += bullet.obj.vel[1] * dt;
            }
            bullet.lifetime -= dt;
            if bullet.lifetime <= 0.0 { bullet.obj.active = false; }
        }
        self.enemy_bullets.retain(|b| b.obj.active &&
            b.obj.pos[0] > -b.obj.radius && b.obj.pos[0] < self.window_size[0] + b.obj.radius &&
            b.obj.pos[1] > -b.obj.radius && b.obj.pos[1] < self.window_size[1] + b.obj.radius
        );
    }

    fn update_ufos(&mut self, dt: f64) {
        let player_pos = self.player.obj.pos;
        let window_width = self.window_size[0];
        let mut new_enemy_bullet_params: Vec<([f64; 2], [f64; 2], f64)> = Vec::new();

        for ufo in self.ufos.iter_mut() {
            ufo.obj.pos[0] += ufo.obj.vel[0] * dt;
            ufo.obj.pos[1] += ufo.obj.vel[1] * dt;

            ufo.change_dir_timer -= dt;
            if ufo.change_dir_timer <= 0.0 {
                let new_y_vel = match self.rng.gen_range(0..3) {
                    0 => UFO_SPEED * 0.5,
                    1 => -UFO_SPEED * 0.5,
                    _ => 0.0,
                };
                ufo.obj.vel[1] = new_y_vel;
                ufo.change_dir_timer = self.rng.gen_range(2.0..5.0);
            }

            if (ufo.obj.pos[1] < ufo.obj.radius && ufo.obj.vel[1] < 0.0) ||
               (ufo.obj.pos[1] > self.window_size[1] - ufo.obj.radius && ufo.obj.vel[1] > 0.0) {
                ufo.obj.vel[1] *= -1.0;
            }

            ufo.shoot_timer -= dt;
            if ufo.shoot_timer <= 0.0 && ufo.obj.pos[0] > 0.0 && ufo.obj.pos[0] < window_width {
                let ufo_pos = ufo.obj.pos;
                let mut dir_to_player = [
                    player_pos[0] - ufo_pos[0],
                    player_pos[1] - ufo_pos[1],
                ];
                normalize_vector(&mut dir_to_player);

                let bullet_p = [
                    ufo_pos[0] + dir_to_player[0] * (ufo.obj.radius + ENEMY_BULLET_RADIUS + 2.0),
                    ufo_pos[1] + dir_to_player[1] * (ufo.obj.radius + ENEMY_BULLET_RADIUS + 2.0),
                ];
                let bullet_v = [
                    dir_to_player[0] * ENEMY_BULLET_SPEED,
                    dir_to_player[1] * ENEMY_BULLET_SPEED,
                ];
                let bullet_r = dir_to_player[1].atan2(dir_to_player[0]);
                new_enemy_bullet_params.push((bullet_p, bullet_v, bullet_r));
                ufo.shoot_timer = UFO_SHOOT_COOLDOWN;
            }

            if ufo.obj.pos[0] < -ufo.obj.radius * 2.0 || ufo.obj.pos[0] > self.window_size[0] + ufo.obj.radius * 2.0 {
                ufo.obj.active = false;
            }
        }

        for (pos, vel, rot) in new_enemy_bullet_params {
            self.enemy_bullets.push(EnemyBullet {
                obj: GameObject {
                    pos,
                    vel,
                    rot,
                    radius: ENEMY_BULLET_RADIUS,
                    active: true,
                },
                lifetime: ENEMY_BULLET_LIFETIME,
            });
            self.assets.play_sound(&self.mixer, "ufo_shoot");
        }

        self.ufos.retain(|u| u.obj.active);
    }

    fn update_particles(&mut self, dt: f64) {
        for particle in &mut self.particles {
            particle.pos[0] += particle.vel[0] * dt;
            particle.pos[1] += particle.vel[1] * dt;
            particle.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);
        
        for line in &mut self.speed_lines {
            line.lifetime -= dt;
        }
        self.speed_lines.retain(|l| l.lifetime > 0.0);		
    }

    fn update_stars(&mut self, _dt: f64) {
		//no op
    }

    fn handle_collisions(&mut self) {
        let mut asteroids_to_break_indices = Vec::new();
        let mut player_bullets_collided_indices = Vec::new();
		let mut saucer_hit_positions = Vec::new();
		
        // Check bullets vs flying saucer collision box
        if let Some(ref mut saucer) = self.flying_saucer {
            for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
                let dx = bullet.obj.pos[0] - saucer.x;
                let dy = bullet.obj.pos[1] - saucer.y;
                let dist_sq = dx * dx + dy * dy;
                if dist_sq < 10000.0 { // 100 pixel radius
                    saucer.take_damage();
                    player_bullets_collided_indices.push(bullet_idx);
					saucer_hit_positions.push(bullet.obj.pos);	
					self.assets.play_sound(&self.mixer, "shoot");
                }
            }
        }	

		for pos in saucer_hit_positions {
			self.spawn_particles(pos, 15, [0.0, 1.0, 1.0, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 3.0);
		}

        for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
            for (asteroid_idx, asteroid) in self.asteroids.iter().enumerate() {
                if distance_sq(bullet.obj.pos, asteroid.obj.pos) < (bullet.obj.radius + asteroid.obj.radius).powi(2) {
                    if !asteroids_to_break_indices.contains(&asteroid_idx) {
                        asteroids_to_break_indices.push(asteroid_idx);
                    }
                    player_bullets_collided_indices.push(bullet_idx);
					self.assets.play_sound(&self.mixer, "shoot");
                    break;
                }
            }
        }
        player_bullets_collided_indices.sort_unstable();
        player_bullets_collided_indices.dedup();
        for i in player_bullets_collided_indices.iter().rev() {
            if *i < self.bullets.len() { self.bullets.remove(*i); }
        }
        asteroids_to_break_indices.sort_unstable_by(|a: &usize, b: &usize| b.cmp(a));
        for index in asteroids_to_break_indices {
            if index < self.asteroids.len() { self.break_asteroid(index); }
        }

        if self.player.invincible_timer <= 0.0 {
            let mut player_collided_with_asteroid = false;
            for asteroid in &self.asteroids {
                if distance_sq(self.player.obj.pos, asteroid.obj.pos) < (self.player.obj.radius + asteroid.obj.radius).powi(2) {
                    player_collided_with_asteroid = true;
                    break;
                }
            }
            if player_collided_with_asteroid {
                self.player_hit();
            }
        }

        let mut ufos_to_destroy_indices = Vec::new();
        player_bullets_collided_indices = Vec::new();

        for (bullet_idx, bullet) in self.bullets.iter().enumerate() {
            for (ufo_idx, ufo) in self.ufos.iter().enumerate() {
                 if distance_sq(bullet.obj.pos, ufo.obj.pos) < (bullet.obj.radius + ufo.obj.radius).powi(2) {
                    if !ufos_to_destroy_indices.contains(&ufo_idx) {
                        ufos_to_destroy_indices.push(ufo_idx);
                    }
                    player_bullets_collided_indices.push(bullet_idx);
                    break;
                 }
            }
        }
        player_bullets_collided_indices.sort_unstable();
        player_bullets_collided_indices.dedup();
        for i in player_bullets_collided_indices.iter().rev() {
            if *i < self.bullets.len() { self.bullets.remove(*i); }
        }
        ufos_to_destroy_indices.sort_unstable_by(|a: &usize,b: &usize| b.cmp(a));
        for ufo_idx in ufos_to_destroy_indices {
            if ufo_idx < self.ufos.len() {
                self.spawn_particles(self.ufos[ufo_idx].obj.pos, PARTICLE_COUNT_UFO, [0.2, 0.8, 0.2, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 3.0);
                self.ufos.remove(ufo_idx);
                self.player.score += UFO_POINTS;
                self.assets.play_sound(&self.mixer, "ufo_explode");
            }
        }

        if self.player.invincible_timer <= 0.0 {
            let mut enemy_bullets_collided_indices = Vec::new();
            let mut player_hit_by_enemy_bullet = false;
            for (bullet_idx, enemy_bullet) in self.enemy_bullets.iter().enumerate() {
                if distance_sq(self.player.obj.pos, enemy_bullet.obj.pos) < (self.player.obj.radius + enemy_bullet.obj.radius).powi(2) {
                    enemy_bullets_collided_indices.push(bullet_idx);
                    player_hit_by_enemy_bullet = true;
                    break;
                }
            }
            if player_hit_by_enemy_bullet {
                 self.player_hit();
            }
            enemy_bullets_collided_indices.sort_unstable_by(|a: &usize,b: &usize| b.cmp(a));
            for i in enemy_bullets_collided_indices {
                 if i < self.enemy_bullets.len() { self.enemy_bullets.remove(i); }
            }
        }

        if self.player.invincible_timer <= 0.0 && !self.game_over {
            let mut ufo_player_collisions: Vec<(usize, [f64; 2])> = Vec::new();
            let mut player_collided_with_ufo_directly = false;

            for (idx, ufo) in self.ufos.iter().enumerate() {
                if distance_sq(self.player.obj.pos, ufo.obj.pos) < (self.player.obj.radius + ufo.obj.radius).powi(2) {
                    ufo_player_collisions.push((idx, ufo.obj.pos));
                    player_collided_with_ufo_directly = true;
                }
            }

            if player_collided_with_ufo_directly {
                self.player_hit();

                ufo_player_collisions.sort_by_key(|k| std::cmp::Reverse(k.0));

                for (idx, ufo_pos) in ufo_player_collisions {
                    self.spawn_particles(ufo_pos, PARTICLE_COUNT_UFO, [0.2, 0.8, 0.2, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 3.0);
                    self.assets.play_sound(&self.mixer, "ufo_explode");
                    if idx < self.ufos.len() {
                        self.ufos.remove(idx);
                    }
                }
            }
        }
    }
	
    fn handle_rush_collisions(&mut self) {
        let mut asteroids_to_break_indices = Vec::new();
        
        for (i, asteroid) in self.asteroids.iter().enumerate() {
            if distance_sq(self.player.obj.pos, asteroid.obj.pos) < (self.player.obj.radius + asteroid.obj.radius).powi(2) {
                asteroids_to_break_indices.push(i);
                self.assets.play_sound(&self.mixer, "explode_asteroid");
            }
        }
        
        asteroids_to_break_indices.sort_unstable_by(|a, b| b.cmp(a));
        asteroids_to_break_indices.dedup();
        
        for index in asteroids_to_break_indices {
            if index < self.asteroids.len() {
                self.break_asteroid(index);
            }
        }
 
        // Ufo collision during rush
        let mut ufos_to_destroy = Vec::new();
        for (i, ufo) in self.ufos.iter().enumerate() {
            if distance_sq(self.player.obj.pos, ufo.obj.pos) < (self.player.obj.radius + ufo.obj.radius).powi(2) {
                ufos_to_destroy.push(i);
            }
        }
        
        ufos_to_destroy.sort_unstable_by(|a, b| b.cmp(a));
        for index in ufos_to_destroy {
            if index < self.ufos.len() {
                self.spawn_particles(self.ufos[index].obj.pos, PARTICLE_COUNT_UFO, [0.2, 0.8, 0.2, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 3.0);
                self.ufos.remove(index);
                self.player.score += UFO_POINTS;
                self.assets.play_sound(&self.mixer, "ufo_explode");
            }
        }
 
        // Boss collision during rush
        if let Some(ref mut saucer) = self.flying_saucer {
            let dx = self.player.obj.pos[0] - saucer.x;
            let dy = self.player.obj.pos[1] - saucer.y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < 5000.0 { // 100 pixel radius approx
                saucer.take_damage();
                // Bounce back slightly
                self.player.obj.pos[0] -= self.player.obj.rot.cos() * 50.0;
                self.player.obj.pos[1] -= self.player.obj.rot.sin() * 50.0;
                self.player.rush_active = false; // End rush on boss hit
                self.spawn_particles(self.player.obj.pos, 15, [0.0, 1.0, 1.0, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 3.0);
            }
        }
    }	

    fn player_hit(&mut self) {
        if self.player.invincible_timer > 0.0 { return; }

        self.assets.play_sound(&self.mixer, "explode_ship");
        self.spawn_particles(self.player.obj.pos, PARTICLE_COUNT_SHIP, [0.8, 0.8, 1.0, 1.0], (PARTICLE_SPEED_MIN, PARTICLE_SPEED_MAX), (PARTICLE_LIFETIME_MIN, PARTICLE_LIFETIME_MAX), 4.0);

        self.player.shields -= 1;
        if self.player.shields == 0 {
			// Play sound again explicitly for death guarantee
			self.assets.play_sound(&self.mixer, "explode_ship"); 
            self.game_over = true;
			self.death_cause = Some(FirmamentDeathCause::Standard);
            if !self.thrust_sink.is_paused() { self.thrust_sink.pause(); }
            debug_print(&format!("Game Over! Score: {}", self.player.score));
        } else {
            self.player.invincible_timer = SHIP_INVINCIBILITY_DURATION; // Just reset timer, keep position
            debug_print("Player respawned");
        }
    }

    fn render_particles(&self, c: Context, g: &mut G2d) { // Piston Context, G2d
        for particle in &self.particles {
            let alpha_f64 = particle.lifetime / particle.max_lifetime;
            let color_alpha = particle.color[3] * alpha_f64 as f32;
            let color = [particle.color[0], particle.color[1], particle.color[2], color_alpha];
            let size = particle.size * alpha_f64.max(0.2);
            piston_window::ellipse( // Using qualified path for clarity, or rely on `use piston_window::*`
                color,
                [particle.pos[0] - size / 2.0, particle.pos[1] - size / 2.0, size, size],
                c.transform,
                g,
            );
        }
		
        for line in &self.speed_lines {
            let alpha = (line.lifetime / line.max_lifetime) as f32;
            let mut color = line.color;
            color[3] *= alpha;
            
            piston_window::line(
                color,
                line.width,
                [line.start_pos[0], line.start_pos[1], line.end_pos[0], line.end_pos[1]],
                c.transform,
                g,
            );
        }		
    }
/*
    fn render_stars(&self, c: Context, g: &mut G2d) {
        for star in &self.stars {
            piston_window::ellipse(
                star.color,
                [star.pos[0] - star.size / 2.0, star.pos[1] - star.size / 2.0, star.size, star.size],
                c.transform,
                g,
            );
        }
    }
*/
    // --- Public API Methods for Game Interaction ---

    /// Updates the game state by a time delta `dt`.
    pub fn update(&mut self, dt: f64) {
        if !self.warnings.is_empty() {
            for (_, lifetime) in self.warnings.iter_mut() {
                *lifetime -= dt;
            }
            self.warnings.retain(|(_, lifetime)| *lifetime > 0.0);
        }		
		
        if self.is_paused {
            // In pause/practice mode: update only player inputs and visual/audio feedback.
            // No movement, no enemy logic, no collisions.
            self.update_player(dt);
            self.update_particles(dt);
            self.update_bullets(dt);
            self.update_enemy_bullets(dt);
            // Ensure thrust sound stops if not thrusting, as the main loop won't handle it.
            if !self.player.is_thrusting && !self.thrust_sink.is_paused() {
                self.thrust_sink.pause();
            }
            return;
        }
        // Check if we're in Fort Silo field and should spawn boss
        if self.map_system.current_field_id == map_system::FieldId3D(-25, 25, 0) {
            if self.flying_saucer.is_none() && !self.boss_fight_active {
                self.flying_saucer = Some(FlyingSaucer::new(WINDOW_WIDTH / 2.0, WINDOW_HEIGHT / 4.0));
                self.boss_fight_active = true;
                self.ufos.clear();
                self.enemy_bullets.clear();	
				self.warnings.push_back(("ENTRAPPED BY A GRAVITATIONAL FORCE".to_string(), WARNING_MESSAGE_DURATION));
                debug_print("Flying Saucer boss fight initiated!");
            }
        }

        // Update flying saucer
        if let Some(ref mut saucer) = self.flying_saucer {
            let player_pos = self.player.obj.pos;
            if let Some(new_projectiles) = saucer.update(dt, WINDOW_WIDTH, WINDOW_HEIGHT, player_pos) {
                self.saucer_projectiles.extend(new_projectiles);
				self.assets.play_sound(&self.mixer, "ufo_shoot");
            }
 
            // Check if defeated
            if saucer.is_defeated() {
                self.flying_saucer = None;
                debug_print("Flying Saucer defeated! Boss fight complete.");
				// Keep boss_fight_active = true to prevent respawn
            }
        }

        // Update projectiles
		let mut player_hit_by_saucer_projectile = false;
        for proj in self.saucer_projectiles.iter_mut() {
            proj.update(dt, WINDOW_WIDTH, WINDOW_HEIGHT);
            
            // Check collision with player ship
            let dx = proj.x - self.player.obj.pos[0];
            let dy = proj.y - self.player.obj.pos[1];
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < (self.player.obj.radius + 10.0).powi(2) && proj.active {
               player_hit_by_saucer_projectile = true;
               proj.active = false;
            }
        }
       self.saucer_projectiles.retain(|p| p.active);
       if player_hit_by_saucer_projectile {
           self.player_hit();
           if self.game_over {
               self.death_cause = Some(FirmamentDeathCause::FlyingSaucer);
           }
       }	

        self.frame_count += 1;
        if self.frame_count % 300 == 0 {
            debug_print(&format!("Frame {} - Player pos: {:?}, vel: {:?}, rot: {:.2}",
                self.frame_count, self.player.obj.pos, self.player.obj.vel, self.player.obj.rot));
            debug_print(&format!("Game state: over={}, waiting={}, #asteroids={}, #bullets={}, #ufos={}, #enemy_bullets={}",
                self.game_over, self.waiting_to_start, self.asteroids.len(), self.bullets.len(), self.ufos.len(), self.enemy_bullets.len()));
        }

        if self.game_over || self.waiting_to_start {
            if !self.thrust_sink.is_paused() { self.thrust_sink.pause(); }
            self.update_particles(dt);
            self.update_stars(dt);
            return;
        }

        if !self.boss_fight_active {
            self.ufo_spawn_timer -= dt;
            if self.ufo_spawn_timer <= 0.0 {
                self.spawn_ufo();
                self.ufo_spawn_timer = self.rng.gen_range(UFO_SPAWN_INTERVAL_MIN..UFO_SPAWN_INTERVAL_MAX);
            }
        }

        self.update_stars(dt);
        self.update_player(dt);
        self.update_asteroids(dt);
        self.update_bullets(dt);
        self.update_ufos(dt);
        self.update_enemy_bullets(dt);
        self.update_particles(dt);

        if !self.game_over {
             self.handle_collisions();
        }
    }

    /// Renders the current game state.
    /// Requires Piston drawing context `c`, graphics backend `g`, and loaded glyphs `glyphs`.
    pub fn render(&mut self, c: Context, g: &mut G2d, glyphs: &mut Glyphs) { // Piston Context, G2d, Glyphs
        // Conditionally draw the background image OR clear the screen
        if self.map_system.current_field_id == map_system::FieldId3D(-2, 5, 0) {
            if let Some(bg_texture) = self.assets.get_texture("rocketbay_air") {
                // For the special field, the image IS the background.
                let transform = c.transform.scale(
                    WINDOW_WIDTH / bg_texture.get_width() as f64,
                    WINDOW_HEIGHT / bg_texture.get_height() as f64,
                );
                piston_window::image(bg_texture, transform, g);
            } else {
                // If the texture fails to load, fall back to the default clear color.
                piston_window::clear([0.0, 0.0, 0.05, 1.0], g);
            }
		} else if self.map_system.current_field_id == map_system::FieldId3D(0, 0, 0) {
           if let Some(bg_texture) = self.assets.get_texture("racetrack0_air") {
               let transform = c.transform.scale(
                   WINDOW_WIDTH / bg_texture.get_width() as f64,
                   WINDOW_HEIGHT / bg_texture.get_height() as f64,
               );
               piston_window::image(bg_texture, transform, g);
           } else {
               // Fallback clear color for this specific field if texture fails.
               let bg_color = self.get_current_field_color();
               piston_window::clear(bg_color, g);
           }
        } else if self.map_system.current_field_id == map_system::FieldId3D(-25, 25, 0) {
            if let Some(bg_texture) = self.assets.get_texture("fort_silo_air") {
                let transform = c.transform.scale(
                   WINDOW_WIDTH / bg_texture.get_width() as f64,
                   WINDOW_HEIGHT / bg_texture.get_height() as f64,
                );
                piston_window::image(bg_texture, transform, g);
            } else {
               // Fallback clear color for this specific field if texture fails.
               let bg_color = self.get_current_field_color();
               piston_window::clear(bg_color, g);
            }		   
        } else {
           // For all other fields, clear with a solid color then tile a transparent texture on top.
           let bg_color = self.get_current_field_color();
           piston_window::clear(bg_color, g);

           if !self.assets.ground_textures_air.is_empty() {
				let texture_index = self.get_ground_texture_index();

				let ground_texture = &self.assets.ground_textures_air[texture_index];
				let tex_width = ground_texture.get_width() as f64;
				let tex_height = ground_texture.get_height() as f64;

               if tex_width > 0.0 && tex_height > 0.0 {
                   // This tiling is in screen space, not world space, because FIRMAMENT has no camera.
                   let mut y = 0.0;
                   while y < WINDOW_HEIGHT {
                       let mut x = 0.0;
                       while x < WINDOW_WIDTH {
                           // The transform here is just a translation in screen space.
                           piston_window::image(ground_texture, c.transform.trans(x, y), g);
                           x += tex_width;
                       }
                       y += tex_height;
                   }
               }
           }
        }

        

        if let Some(asteroid_texture) = self.assets.get_texture("asteroid") {
            for asteroid in &self.asteroids {
                let radius = asteroid.obj.radius;
                let tex_w = asteroid_texture.get_width() as f64;
                let tex_h = asteroid_texture.get_height() as f64;

                if tex_w > 0.0 && tex_h > 0.0 {
                    let base_transform = c.transform
                        .trans(asteroid.obj.pos[0], asteroid.obj.pos[1])
                        .rot_rad(asteroid.obj.rot)
                        .trans(-radius, -radius);

                    let scale_factor_x = (radius * 2.0) / tex_w;
                    let scale_factor_y = (radius * 2.0) / tex_h;
                    let final_transform = base_transform.scale(scale_factor_x, scale_factor_y);
                    piston_window::image(asteroid_texture, final_transform, g); // piston_window::image
                }
            }
        } else {
            let asteroid_color = [0.5, 0.3, 0.1, 1.0];
            for asteroid in &self.asteroids {
                piston_window::ellipse(asteroid_color, [asteroid.obj.pos[0] - asteroid.obj.radius, asteroid.obj.pos[1] - asteroid.obj.radius, asteroid.obj.radius * 2.0, asteroid.obj.radius * 2.0,], c.transform, g);
            }
        }

        if let Some(ufo_texture) = self.assets.get_texture("ufo") {
            for ufo in &self.ufos {
                let radius = ufo.obj.radius;
                let tex_w = ufo_texture.get_width() as f64;
                let tex_h = ufo_texture.get_height() as f64;

                if tex_w > 0.0 && tex_h > 0.0 {
                    let base_transform = c.transform
                        .trans(ufo.obj.pos[0], ufo.obj.pos[1])
                        .trans(-radius, -radius);

                    let scale_factor_x = (radius * 2.0) / tex_w;
                    let scale_factor_y = (radius * 2.0) / tex_h;
                    let final_transform = base_transform.scale(scale_factor_x, scale_factor_y);
                    piston_window::image(ufo_texture, final_transform, g);
                }
            }
        } else {
            let ufo_color = [0.2, 0.8, 0.2, 1.0];
            for ufo in &self.ufos {
                 piston_window::ellipse(ufo_color, [ufo.obj.pos[0] - ufo.obj.radius, ufo.obj.pos[1] - ufo.obj.radius, ufo.obj.radius * 2.0, ufo.obj.radius * 2.0], c.transform, g);
                 piston_window::rectangle([0.3,0.9,0.3,1.0], [ufo.obj.pos[0] - ufo.obj.radius * 1.2, ufo.obj.pos[1] - ufo.obj.radius * 0.2, ufo.obj.radius * 2.4, ufo.obj.radius * 0.4], c.transform, g);
            }
        }
	   
		if let Some(bullet_texture) = self.assets.get_texture("plasma_blast") {
            for bullet in &self.bullets {
                let tex_w = bullet_texture.get_width() as f64;
                let tex_h = bullet_texture.get_height() as f64;

                if tex_w > 0.0 && tex_h > 0.0 {
                     let transform = c.transform
                        .trans(bullet.obj.pos[0], bullet.obj.pos[1])
                        .rot_rad(bullet.obj.rot)
                        .trans(-tex_w / 2.0, -tex_h / 2.0);

                    piston_window::image(bullet_texture, transform, g);
                }
            }
        } else {
            let bullet_color = [1.0, 1.0, 0.5, 1.0];
            for bullet in &self.bullets {
                piston_window::ellipse(bullet_color, [bullet.obj.pos[0] - bullet.obj.radius, bullet.obj.pos[1] - bullet.obj.radius, bullet.obj.radius * 2.0, bullet.obj.radius * 2.0], c.transform, g);
            }
        }

        let enemy_bullet_color = [1.0, 0.3, 0.3, 1.0];
        for bullet in &self.enemy_bullets {
            piston_window::ellipse(enemy_bullet_color, [bullet.obj.pos[0] - bullet.obj.radius, bullet.obj.pos[1] - bullet.obj.radius, bullet.obj.radius * 2.0, bullet.obj.radius * 2.0], c.transform, g);
        }
		
        // Draw flying saucer and projectiles
        if let Some(ref saucer) = self.flying_saucer {
            if let Some(saucer_tex) = self.assets.get_texture("flying_saucer") {
                saucer.draw(c, g, saucer_tex);
            }
        }
		
		self.render_particles(c, g);

        for proj in &self.saucer_projectiles {
            proj.draw(c, g, self.assets.get_texture("pulse_orb"));
        }	

        let player_blinks = self.player.invincible_timer > 0.0 && (self.player.invincible_timer * 12.0) as i32 % 2 == 0;
        if (!self.game_over || self.is_paused) && (self.player.invincible_timer <= 0.0 || !player_blinks) {
            if let Some(ship_texture) = self.assets.get_texture("fighter_jet") {
                let desired_size = self.player.obj.radius * 2.0;
                let tex_w = ship_texture.get_width() as f64;
                let tex_h = ship_texture.get_height() as f64;

                if tex_w > 0.0 && tex_h > 0.0 {
                    let base_transform = c.transform
                        .trans(self.player.obj.pos[0], self.player.obj.pos[1])
                        .rot_rad(self.player.obj.rot + std::f64::consts::FRAC_PI_2)
                        .trans(-desired_size / 2.0, -desired_size / 2.0);

                    let scale_factor_x = desired_size / tex_w;
                    let scale_factor_y = desired_size / tex_h;
                    let final_ship_transform = base_transform.scale(scale_factor_x, scale_factor_y);
                    piston_window::image(ship_texture, final_ship_transform, g);
                } else {
                    debug_print("Ship texture has zero dimensions, drawing fallback polygon.");
                    let local_radius = self.player.obj.radius;
                    let player_color = [0.7, 0.7, 1.0, 1.0];
                    let points = [[local_radius, 0.0], [-local_radius * 0.7, -local_radius * 0.8], [-local_radius * 0.7, local_radius * 0.8]];
                    piston_window::polygon(player_color, &points, c.transform.trans(self.player.obj.pos[0], self.player.obj.pos[1]).rot_rad(self.player.obj.rot), g);
                }
            } else {
                let radius = self.player.obj.radius;
                let player_color = [0.7, 0.7, 1.0, 1.0];
                let points = [[radius, 0.0], [-radius * 0.7, -radius * 0.8], [-radius * 0.7, radius * 0.8]];
                piston_window::polygon(player_color, &points, c.transform.trans(self.player.obj.pos[0], self.player.obj.pos[1]).rot_rad(self.player.obj.rot), g);
            }

            if self.player.is_thrusting {
                let flame_color = [1.0, self.rng.gen_range(0.3..0.7), 0.0, 0.9];
                let radius = self.player.obj.radius;
                let flame_length = radius * self.rng.gen_range(1.2..1.8);
                let flame_width = radius * 0.7;
                let flame_points = [ [-radius * 0.7, -flame_width * 0.5], [-radius * 0.7 - flame_length, 0.0], [-radius * 0.7, flame_width * 0.5]];
                piston_window::polygon(flame_color, &flame_points, c.transform.trans(self.player.obj.pos[0], self.player.obj.pos[1]).rot_rad(self.player.obj.rot), g);
            }
        }

        let white = [1.0, 1.0, 1.0, 1.0];
        let red = [1.0, 0.2, 0.2, 1.0];
        let green = [0.0, 1.0, 0.0, 1.0];

        let field_font_size = 14;
        let score_font_size = 20;

        let bg_color = [0.1, 0.1, 0.1, 0.8]; // Dark Gray/Black semi-transparent		

        let field_text_y_baseline = 10.0 + field_font_size as f64;
        let score_text_y_baseline = field_text_y_baseline + 5.0 + score_font_size as f64;

        // --- FIRMAMENT Field Text with Background ---
        let field_display_text = if self.map_system.current_field_id == map_system::FieldId3D(0, 0, 0) {
            format!("FIRMAMENT_field.x[0]y[0]z[0] RACETRACK")
        } else if self.map_system.current_field_id == map_system::FieldId3D(-2, 5, 0) {
            format!("FIRMAMENT_field.x[-2]y[5]z[0] ROCKETBAY")
        } else if self.map_system.current_field_id == map_system::FieldId3D(-25, 25, 0) {
            format!("FIRMAMENT_field.x[{}]y[{}]z[{}] FORT SILO", 
                self.map_system.current_field_id.0,
                self.map_system.current_field_id.1, 
                self.map_system.current_field_id.2)
        } else {
            self.map_system.get_display_string()
        };
		
        let field_text_x_pos = 10.0; // X position for the text

        // Calculate text width for the background
        let text_width = match glyphs.width(field_font_size, &field_display_text) {
            Ok(w) => w,
            Err(_) => field_display_text.chars().count() as f64 * (field_font_size as f64 * 0.6), // Fallback width
        };
        let padding = 2.0;
        let bg_rect_x = field_text_x_pos - padding;
        // Text is drawn with y as baseline. Top of text is roughly `baseline - font_size`.
        let bg_rect_y = field_text_y_baseline - field_font_size as f64 - padding;
        let bg_rect_width = text_width + 2.0 * padding;
        let bg_rect_height = field_font_size as f64 + 2.0 * padding;
        let black_color = [0.0, 0.0, 0.0, 1.0]; // Black background

        piston_window::rectangle(
            black_color,
            [bg_rect_x, bg_rect_y, bg_rect_width, bg_rect_height],
            c.transform, // Using the same transform as the text
            g,
        );
        piston_window::text::Text::new_color(white, field_font_size).draw(
            &field_display_text, glyphs, &c.draw_state,
            c.transform.trans(field_text_x_pos, field_text_y_baseline), g
        ).ok();
		
		// --- SCORE: Background and Text ---
        let score_text = format!("{}", self.player.score);
        let score_text_width = glyphs.width(score_font_size, &score_text).unwrap_or(0.0);
        let score_bg_x = 10.0 - padding;
        let score_bg_y = score_text_y_baseline - score_font_size as f64 - padding;
        let score_bg_width = score_text_width + 2.0 * padding;
        let score_bg_height = score_font_size as f64 + 2.0 * padding;
 
        piston_window::rectangle(
            bg_color,
            [score_bg_x, score_bg_y, score_bg_width, score_bg_height],
            c.transform,
            g,
        );		
        piston_window::text::Text::new_color(green, score_font_size).draw(
            &score_text, glyphs, &c.draw_state,
            c.transform.trans(10.0, score_text_y_baseline), g
        ).ok();

		// --- SHIELDS: Background and Text ---
        let shields_text = format!("SHIELDS: {}", self.player.shields);
        let shields_text_width = glyphs.width(score_font_size, &shields_text).unwrap_or(0.0);
        let shields_text_x_pos = self.window_size[0] - 750.0;
        let shields_bg_x = shields_text_x_pos - padding;
        let shields_bg_y = score_text_y_baseline - score_font_size as f64 - padding;
        let shields_bg_width = shields_text_width + 2.0 * padding;
        let shields_bg_height = score_font_size as f64 + 2.0 * padding;
 
        piston_window::rectangle(
            bg_color,
            [shields_bg_x, shields_bg_y, shields_bg_width, shields_bg_height],
            c.transform,
            g,
        );		
        piston_window::text::Text::new_color(green, score_font_size).draw(
            &shields_text, glyphs, &c.draw_state,
            c.transform.trans(self.window_size[0] - 750.0, score_text_y_baseline), g // WAS - 100.0
        ).ok();

        if self.game_over && !self.is_paused {
            piston_window::text::Text::new_color(red, 40).draw("GAME OVER", glyphs, &c.draw_state, c.transform.trans(self.window_size[0] / 2.0 - 120.0, self.window_size[1] / 2.0 - 20.0), g).ok();
            piston_window::text::Text::new_color(white, 20).draw("Press ENTER to Restart", glyphs, &c.draw_state, c.transform.trans(self.window_size[0] / 2.0 - 115.0, self.window_size[1] / 2.0 + 30.0), g).ok();
        }
		
        if self.is_paused {
            if let Some(inputs_texture) = self.assets.get_texture("inputs") {
                let base_x = 600.0;
                let base_y = 1075.0;
                let texture_height = inputs_texture.get_height() as f64;
                piston_window::image(
                    inputs_texture,
                    c.transform.trans(base_x, base_y - texture_height),
                    g,
                );
            }
        }		
		
        if let Some((text, _lifetime)) = self.warnings.front() {
            let font_size = 20;
            let orange = [1.0, 0.5, 0.0, 1.0];
 
            let text_width = match glyphs.width(font_size, text) {
                Ok(w) => w,
                Err(_) => text.len() as f64 * (font_size as f64 * 0.6), // Fallback
            };
 
            let screen_width = self.window_size[0];
            let screen_height = self.window_size[1];
 
            let padding = 15.0;
            let bg_width = text_width + (padding * 2.0);
            let bg_height = font_size as f64 + (padding * 2.0);
 
            let bg_x = (screen_width - bg_width) / 2.0;
            let bg_y = screen_height / 4.0;
 
            let text_x = bg_x + padding;
            let text_y_baseline = bg_y + padding + font_size as f64;
 
            let bg_color = [0.0, 0.0, 0.0, 0.5]; // Black with 50% opacity
 
            piston_window::rectangle(
                bg_color,
                [bg_x, bg_y, bg_width, bg_height],
                c.transform,
                g,
            );
 
            piston_window::text::Text::new_color(orange, font_size)
                .draw(
                    text,
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(text_x, text_y_baseline),
                    g,
                )
                .ok();
        }		
    }

    /// Handles key press events.
    pub fn key_pressed(&mut self, key: Key) { // piston_window::Key
        if self.is_paused {
            return;
        }
        match key {
            Key::Up | Key::W => self.player.is_thrusting = true,
			Key::LShift | Key::RShift => self.player.is_braking = true,
            Key::Left | Key::A => self.player.rotating_left = true,
            Key::Right | Key::D => self.player.rotating_right = true,
            Key::Space => {
                if self.player.rush_cooldown <= 0.0 && !self.player.rush_active {
                    self.player.rush_active = true;
                    self.player.rush_timer = SHIP_RUSH_DURATION;
                    self.player.rush_cooldown = SHIP_RUSH_COOLDOWN;
                    self.assets.play_sound(&self.mixer, "shoot"); // Re-using thrust or add rush sound
                }
            },
            Key::Return => {
                // Restarting the game only works if not paused.
                if self.game_over && !self.is_paused {
                    self.reset_game_state();
                }
            },
			Key::T => self.task_bar_open = !self.task_bar_open,
            _ => {}
        }
    }

    /// Handles key release events.
    pub fn key_released(&mut self, key: Key) { // piston_window::Key
       if self.is_paused {
            return;
        }	
	
        match key {
            Key::Up | Key::W => self.player.is_thrusting = false,
			Key::LShift | Key::RShift => self.player.is_braking = false,
            Key::Left | Key::A => self.player.rotating_left = false,
            Key::Right | Key::D => self.player.rotating_right = false,
            _ => {}
        }
    }

    // --- Public Getters for Game State ---
    pub fn get_score(&self) -> u32 { self.player.score }
    pub fn get_shields(&self) -> u32 { self.player.shields }
    pub fn is_game_over(&self) -> bool { self.game_over }
    pub fn is_waiting_to_start(&self) -> bool { self.waiting_to_start }
    pub fn get_field_id_display_string(&self) -> String { self.map_system.get_display_string() }
    pub fn get_current_field_id(&self) -> map_system::FieldId3D { self.map_system.current_field_id }
	pub fn is_boss_fight_active(&self) -> bool { self.boss_fight_active }
	pub fn is_boss_defeated(&self) -> bool {
		self.boss_fight_active && self.flying_saucer.is_none()
	}	
	
    pub fn mouse_pressed(&mut self, button: MouseButton) {
       if self.is_paused {
            return;
        }		
		
        if button == MouseButton::Left {
            self.shoot_player_bullet();
        }
    }	
}