// utils/animation_queue.rs

/// A queue of animations with durations to be played in sequence
pub struct AnimationQueue {
    pub animations: Vec<(String, f64)>, // (animation_name, duration)
    pub current_index: usize,
}

impl AnimationQueue {
    /// Creates a new empty animation queue
    pub fn new() -> Self {
        AnimationQueue {
            animations: Vec::new(),
            current_index: 0,
        }
    }

    /// Adds an animation to the queue if it's different from the last one
    pub fn add_animation(&mut self, name: &str, duration: f64) {
        // Only add if different from the last one
		if self.animations.last().map_or(true, |last| last.0 != name) {
			self.animations.push((name.to_string(), duration));
        }
    }

    /// Updates the animation queue with the given delta time
    /// Returns true if an animation is currently active, false otherwise
    pub fn update(&mut self, dt: f64) -> bool {
        if self.animations.is_empty() {
            return false;
        }

        let (_, ref mut duration) = self.animations[self.current_index];
        *duration -= dt;

        if *duration <= 0.0 {
            self.current_index += 1;
            if self.current_index >= self.animations.len() {
                self.animations.clear();
                self.current_index = 0;
                return false;
            }
        }

        true
    }

    /// Gets the name of the current animation, if any
    pub fn get_current_animation(&self) -> Option<&str> {
        if self.animations.is_empty() {
            None
        } else {
            Some(&self.animations[self.current_index].0)
        }
    }
}
