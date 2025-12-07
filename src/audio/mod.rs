// audio/mod.rs - FIXED FOR RODIO 0.21.1 OGG SUPPORT (v2)

use rodio::{source::Source, Decoder, OutputStream, Sink};
use rodio::mixer::Mixer;
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub struct AudioManager {
    _stream: OutputStream,
    mixer: Arc<Mixer>,
    sound_effects: Arc<Mutex<HashMap<String, PathBuf>>>,
}

impl AudioManager {
    pub fn new() -> Result<Self, String> {
        println!("[AudioManager] Initializing audio system...");
        
        let stream = rodio::OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to open audio stream: {}", e))?;

        let mixer = stream.mixer().clone();
        
        println!("[AudioManager] Audio system initialized successfully");

        Ok(AudioManager {
            _stream: stream,
            mixer: mixer.into(),
            sound_effects: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Play looping background music
    pub fn play_looping_sound(&self, path: &PathBuf) -> Result<Sink, String> {
        println!("[AudioManager] play_looping_sound called for: {:?}", path);
        
        // Check if file exists
        if !path.exists() {
            return Err(format!("Audio file does not exist: {:?}", path));
        }
        
        let file = File::open(path)
            .map_err(|e| format!("Failed to open audio file {:?}: {}", path, e))?;
        
        println!("[AudioManager] File opened successfully, attempting to decode...");

        // Use try_from for rodio 0.21.1 - handles format detection properly
        let source = Decoder::try_from(file)
            .map_err(|e| format!("Failed to decode audio file {:?}: {}", path, e))?;
        
        // Log audio properties
        println!("[AudioManager] Decoded successfully!");
        println!("[AudioManager]   Sample rate: {:?}", source.sample_rate());
        println!("[AudioManager]   Channels: {:?}", source.channels());
        
        let looped_source = source.repeat_infinite();

        let sink = Sink::connect_new(&self.mixer);
        sink.set_volume(1.0); // Ensure volume is at max
        sink.append(looped_source);
        
        // Sink should already be playing, but let's be explicit
        if sink.is_paused() {
            println!("[AudioManager] Sink was paused, unpausing...");
            sink.play();
        }
        
        println!("[AudioManager] Looping sound started, sink empty: {}, paused: {}", 
                 sink.empty(), sink.is_paused());

        Ok(sink)
    }

    /// Load a sound effect and associate it with a name
    pub fn load_sound_effect(&self, name: &str, path: &PathBuf) -> Result<(), String> {
        println!("[AudioManager] Loading sound effect '{}' from {:?}", name, path);
        
        // Check if file exists first
        if !path.exists() {
            return Err(format!("Sound file does not exist: {:?}", path));
        }
        
        let file = File::open(path).map_err(|e| {
            format!("Failed to open sound effect file {}: {}", path.display(), e)
        })?;

        // Test decode with try_from
        let decoder = Decoder::try_from(file).map_err(|e| {
            format!(
                "Failed to decode sound effect file {}: {}",
                path.display(),
                e
            )
        })?;
        
        println!("[AudioManager] '{}' decoded OK - sample_rate: {}, channels: {}", 
                 name, decoder.sample_rate(), decoder.channels());

        // If we got here, the sound file is valid
        let mut effects = self.sound_effects.lock().unwrap();
        effects.insert(name.to_string(), path.clone());

        Ok(())
    }

    /// Play a sound effect by name
    pub fn play_sound_effect(&self, name: &str) -> Result<(), String> {
        // Debounce logic
        static mut LAST_PLAYED: Option<std::collections::HashMap<String, std::time::Instant>> = None;
        static ONCE: std::sync::Once = std::sync::Once::new();

        unsafe {
            ONCE.call_once(|| {
                LAST_PLAYED = Some(std::collections::HashMap::new());
            });

            if let Some(ref mut last_played) = LAST_PLAYED {
                let now = std::time::Instant::now();
                if let Some(last_time) = last_played.get(name) {
                    let elapsed = now.duration_since(*last_time);
                    if elapsed.as_millis() < 50 {
                        return Ok(());
                    }
                }
                last_played.insert(name.to_string(), now);
            }
        }

        // Clone path and release lock before I/O
        let path = {
            let effects = self.sound_effects.lock().unwrap();
            effects
                .get(name)
                .ok_or_else(|| format!("Sound effect '{}' not found", name))?
                .clone()
        };

        let file = File::open(&path)
            .map_err(|e| format!("Failed to open sound effect file: {}", e))?;

        let source = Decoder::try_from(file)
            .map_err(|e| format!("Failed to decode sound effect file: {}", e))?;

        let sink = Sink::connect_new(&self.mixer);
        sink.set_volume(1.0);
        sink.append(source);
        sink.detach();

        Ok(())
    }

    /// Load all sound effects in the sfx directory
    pub fn load_sfx_directory(&self, exe_dir: &std::path::Path) -> Result<(), String> {
        let mut potential_sfx_dirs: Vec<Option<PathBuf>> = Vec::new();

        potential_sfx_dirs.push(Some(Path::new("sfx").to_path_buf()));
        potential_sfx_dirs.push(Some(exe_dir.join("sfx")));
        potential_sfx_dirs.push(exe_dir.parent().map(|p| p.join("sfx")));
        potential_sfx_dirs.push(Some(Path::new(".").join("sfx")));

        println!("Searching for sfx directory in the following locations:");
        for dir_option in &potential_sfx_dirs {
            if let Some(dir) = dir_option {
                println!("  - {:?} (exists: {})", dir.display(), dir.exists());
            }
        }

        let sfx_dir = potential_sfx_dirs
            .into_iter()
            .filter_map(|dir| dir)
            .find(|dir| dir.exists())
            .ok_or_else(|| format!("Could not find sfx directory in any standard location"))?;

        println!("Found sound effects directory at: {:?}", sfx_dir.display());

        let effects = [
			("boost", "boost.wav"),
            ("melee", "slash.wav"),
            ("ranged", "racer_ranged.wav"),
            ("block", "block.wav"),
            ("raise_shield", "guard.wav"),
            ("block_break", "block_break.wav"),
            ("rush", "rush.wav"),
            ("bike_start", "SbrxBike_start.wav"),
            ("title", "title.ogg"),
            ("reload", "reload.wav"),
            ("bike_accelerate", "sbrxBike_accelerate.ogg"),
            ("bike_idle", "sbrxBike_idle.ogg"),
            ("death", "death.wav"),
            ("firearm", "firearm.wav"),
            ("aim", "aim.wav"),
            ("hit", "hit.wav"),
            ("mantis_attack", "SlashSwipe.wav"),
            ("slash_combo", "slashCombo.wav"),
            ("crickets", "crickets.ogg"),
			("pause", "pause.ogg"),
        ];

        for (name, filename) in effects.iter() {
            let path = sfx_dir.join(filename);
            match self.load_sound_effect(name, &path) {
                Ok(_) => println!("✓ Loaded: {} from {}", name, path.display()),
                Err(e) => println!("✗ FAILED to load {}: {}", name, e),
            }
        }

        Ok(())
    }

    /// Play a sound effect in a loop by name and return the Sink for control.
	pub fn play_sfx_loop(&self, name: &str) -> Result<Sink, String> {
		println!("[AudioManager] play_sfx_loop called for: {}", name);
		
		// Clone path and release lock before I/O
		let path = {
			let effects_guard = self.sound_effects.lock().unwrap();
			effects_guard
				.get(name)
				.ok_or_else(|| format!("Looping sound effect '{}' not found", name))?
				.clone()
		};

		println!("[AudioManager] Found path: {:?}", path);

		let file = File::open(&path)
			.map_err(|e| format!("Failed to open audio file for loop {}: {}", name, e))?;

		let source = Decoder::try_from(file)
			.map_err(|e| format!("Failed to decode audio file for loop {}: {}", name, e))?;

		println!("[AudioManager] '{}' decoded - sample_rate: {}, channels: {}", 
				 name, source.sample_rate(), source.channels());

		let looped_source = source.repeat_infinite();

		let sink = Sink::connect_new(&self.mixer);
		sink.set_volume(1.0);
		sink.append(looped_source);
		
		println!("[AudioManager] '{}' sink created - empty: {}, paused: {}", 
				 name, sink.empty(), sink.is_paused());

		// DEBUG: Add this section
		//println!("[AudioManager] Sleeping 100ms to test audio...");
		//std::thread::sleep(std::time::Duration::from_millis(100));
		//println!("[AudioManager] After sleep - sink empty: {}, paused: {}", sink.empty(), sink.is_paused());

		Ok(sink)
	}
	
    /// Play a sound effect once by name and return the Sink for control.
    pub fn play_sfx_with_sink(&self, name: &str) -> Result<Sink, String> {
        let path = {
            let effects_guard = self.sound_effects.lock().unwrap();
            effects_guard
                .get(name)
                .ok_or_else(|| format!("Sound effect '{}' not found", name))?
                .clone()
        };
 
        let file = File::open(&path)
            .map_err(|e| format!("Failed to open audio file {}: {}", name, e))?;
 
        let source = Decoder::try_from(file)
            .map_err(|e| format!("Failed to decode audio file {}: {}", name, e))?;
 
        let sink = Sink::connect_new(&self.mixer);
        sink.set_volume(1.0);
        sink.append(source);
 
        Ok(sink)
    }

 	pub fn play_sfx_with_sink_looped(&self, name: &str) -> Result<Sink, String> {
 		let path = {
 			let effects_guard = self.sound_effects.lock().unwrap();
 			effects_guard
 				.get(name)
 				.ok_or_else(|| format!("Sound effect '{}' not found", name))?
 				.clone()
 		};
  
 		let file = File::open(&path)
 			.map_err(|e| format!("Failed to open audio file {}: {}", name, e))?;
  
 		let source = Decoder::try_from(file)
 			.map_err(|e| format!("Failed to decode audio file {}: {}", name, e))?;
  
 		let sink = Sink::connect_new(&self.mixer);
 		sink.set_volume(1.0);
 		sink.append(source.repeat_infinite());
  
 		Ok(sink)
 	}	

    pub fn find_file_in_multiple_locations(base_filename: &str, exe_dir: &Path) -> Option<PathBuf> {
        let mut potential_locations: Vec<Option<PathBuf>> = Vec::new();

        potential_locations.push(Some(Path::new("assets").join(base_filename)));
        potential_locations.push(Some(exe_dir.join("assets").join(base_filename)));
        potential_locations.push(
            exe_dir
                .parent()
                .map(|p| p.join("assets").join(base_filename)),
        );
        potential_locations.push(Some(exe_dir.join(base_filename)));
        potential_locations.push(exe_dir.parent().map(|p| p.join(base_filename)));
        potential_locations.push(Some(Path::new(".").join(base_filename)));
        potential_locations.push(Some(Path::new("target/debug").join(base_filename)));

        println!(
            "Searching for '{}' in the following locations:",
            base_filename
        );
        for path_option in &potential_locations {
            if let Some(path) = path_option {
                println!("  - {:?} (exists: {})", path.display(), path.exists());
            }
        }

        for path_option in potential_locations {
            if let Some(path) = path_option {
                if path.exists() {
                    println!("Found file '{}' at: {:?}", base_filename, path.display());
                    return Some(path);
                }
            }
        }

        println!(
            "Could not find file '{}' in any standard location",
            base_filename
        );
        None
    }
}
 