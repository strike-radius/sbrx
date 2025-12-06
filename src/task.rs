// File: src/task.rs

use crate::entities::cpu_entity::CpuVariant;
use piston_window::*;

const RED_ORANGE: [f32; 4] = [1.0, 0.27, 0.0, 1.0];
const GREY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];
const LIME_GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

pub struct Task {
    pub description: String,
    pub completed: bool,
}

pub struct TaskSystem {
    tasks: Vec<Task>,
    pub active: bool,
    pub open: bool,
    pub giant_mantis_defeated: u32,
    pub rattlesnake_defeated: u32,
    pub giant_rattlesnake_defeated: u32,
    pub raptor_nest_cleared: bool,
    pub raptor_defeated: u32,
    pub t_rex_defeated: u32,
    pub blood_idol_defeated: u32,
    pub rocketbay_found: bool,
    pub survivors_found: u32,
    pub fort_silo_reached: bool,
    pub flying_saucer_defeated: bool,
    pub landed_on_fort_silo: bool,
    pub void_tempest_defeated: u32,
    pub light_reaver_defeated: u32,
    pub night_reaver_defeated: u32,
    pub razor_fiend_defeated: u32,
    pub grand_commander_spoken_to: bool,
    pub racer_returned_to_racetrack: bool,
	pub auto_close_timer: f64,
}

impl TaskSystem {
    pub fn new() -> Self {
        TaskSystem {
            tasks: Vec::new(),
            active: false, // Start inactive
            open: false,
            giant_mantis_defeated: 0,
            rattlesnake_defeated: 0,
            giant_rattlesnake_defeated: 0,
            raptor_nest_cleared: false,
            raptor_defeated: 0,
            t_rex_defeated: 0,
            blood_idol_defeated: 0,
            void_tempest_defeated: 0,
            light_reaver_defeated: 0,
            night_reaver_defeated: 0,
            razor_fiend_defeated: 0,
            rocketbay_found: false,
            survivors_found: 0,
            fort_silo_reached: false,
            flying_saucer_defeated: false,
            landed_on_fort_silo: false,
            grand_commander_spoken_to: false,
            racer_returned_to_racetrack: false,
			auto_close_timer: 0.0,
        }
    }

    pub fn populate_taskbar1(&mut self) {
        self.add_task("DEFEAT 3 GIANT MANTIS");
        self.add_task("DEFEAT 25 RATTLESNAKE");
        self.add_task("DEFEAT 5 GIANT RATTLESNAKE");
    }

    pub fn populate_taskbar2(&mut self) {
        self.add_task("CLEAR RAPTOR NEST: FIELD[X1 Y0]");
        self.add_task("DEFEAT 10 RAPTOR");
    }

    pub fn populate_taskbar3(&mut self) {
        self.add_task("GET TO THE ROCKETBAY [X-2 Y5]");
    }

    pub fn populate_taskbar4(&mut self) {
        self.add_task("DEFEAT 1 T-REX");
    }

    pub fn add_task(&mut self, description: &str) {
        if !self.tasks.iter().any(|t| t.description == description) {
            self.tasks.push(Task {
                description: description.to_string(),
                completed: false,
            });
            self.open = true;
			self.auto_close_timer = 7.0;
        }
    }

    pub fn increment_kill_count(&mut self, variant: CpuVariant) {
        match variant {
            CpuVariant::GiantMantis => self.giant_mantis_defeated += 1,
            CpuVariant::Rattlesnake => self.rattlesnake_defeated += 1,
            CpuVariant::GiantRattlesnake => self.giant_rattlesnake_defeated += 1,
            CpuVariant::Raptor => self.raptor_defeated += 1,
            CpuVariant::TRex => self.t_rex_defeated += 1,
            CpuVariant::BloodIdol => self.blood_idol_defeated += 1,
            CpuVariant::VoidTempest => self.void_tempest_defeated += 1,
            CpuVariant::LightReaver => self.light_reaver_defeated += 1,
            CpuVariant::NightReaver => self.night_reaver_defeated += 1,
            CpuVariant::RazorFiend => self.razor_fiend_defeated += 1,
        }
    }

    pub fn mark_raptor_nest_cleared(&mut self) {
        self.raptor_nest_cleared = true;
    }

    pub fn mark_rocketbay_found(&mut self) {
        self.rocketbay_found = true;
    }

    pub fn mark_grand_commander_spoken_to(&mut self) {
        self.grand_commander_spoken_to = true;
    }

    pub fn mark_racer_returned_to_racetrack(&mut self) {
        self.racer_returned_to_racetrack = true;
    }

    pub fn mark_proceed_to_starting_line_complete(&mut self) {
        for task in self.tasks.iter_mut() {
            if task.description == "PROCEED TO THE STARTING LINE" {
                if !task.completed {
                    task.completed = true;
                    self.open = true;
					self.auto_close_timer = 7.0;
                }
                break;
            }
        }
    }

    pub fn mark_fighter_jet_as_boarded(&mut self) -> bool {
        let mut just_completed = false;
        for task in self.tasks.iter_mut() {
            if task.description == "BOARD THE FIGHTERJET" {
                if !task.completed {
                    task.completed = true;
                    self.open = true;
					self.auto_close_timer = 7.0;
                    just_completed = true;
                }
                break;
            }
        }
        just_completed
    }

    pub fn mark_fort_silo_reached(&mut self) {
        self.fort_silo_reached = true;
    }

    pub fn has_task(&self, description: &str) -> bool {
        self.tasks.iter().any(|t| t.description == description)
    }

    pub fn is_task_complete(&self, description: &str) -> bool {
        self.tasks
            .iter()
            .any(|t| t.description == description && t.completed)
    }

    pub fn update(&mut self) -> u32 {
        let mut tasks_to_add = Vec::new();
        let mut task_was_completed = false;
        let mut completed_this_frame = 0;

        for task in self.tasks.iter_mut() {
            if task.completed {
                continue;
            }

            let was_complete = !task.completed;

            match task.description.as_str() {
                //taskbar 1
                "DEFEAT 3 GIANT MANTIS" => {
                    if self.giant_mantis_defeated >= 3 {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                "DEFEAT 25 RATTLESNAKE" => {
                    if self.rattlesnake_defeated >= 25 {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                "DEFEAT 5 GIANT RATTLESNAKE" => {
                    if self.giant_rattlesnake_defeated >= 5 {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                //taskbar 2
                // BUG FIX: Changed string to match the one in populate_taskbar2
                "CLEAR RAPTOR NEST: FIELD[X1 Y0]" => {
                    if self.raptor_nest_cleared {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                "DEFEAT 10 RAPTOR" => {
                    if self.raptor_defeated >= 10 {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                //taskbar 3
                "GET TO THE ROCKETBAY [X-2 Y5]" => {
                    if self.rocketbay_found {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                //taskbar 4
                "DEFEAT 1 T-REX" => {
                    if self.t_rex_defeated >= 1 {
                        task.completed = true;
                        task_was_completed = true;
                    }
                }
                "FIND 10 SURVIVORS" => {
                    if self.survivors_found >= 10 {
                        task.completed = true;
                        task_was_completed = true;
                        // Mark tasks to add after the loop
                        tasks_to_add.push("BOARD THE FIGHTERJET");
                        tasks_to_add.push("FLY TO FORT SILO [X-25 Y25]");
                    }
                }
                "BOARD THE FIGHTERJET" => {
                    // This task is completed directly via mark_fighter_jet_boarded()
                    // when the player enters Firmament mode
                }
                "FLY TO FORT SILO [X-25 Y25]" => {
                    if self.fort_silo_reached {
                        task.completed = true;
                        task_was_completed = true;
                        // Add new task after completion
                        tasks_to_add.push("DEFEAT THE FLYING SAUCER");
                    }
                }
                "DEFEAT THE FLYING SAUCER" => {
                    if self.flying_saucer_defeated {
                        task.completed = true;
                        task_was_completed = true;
                        tasks_to_add.push("LAND ON FORT SILO");
                    }
                }
                "LAND ON FORT SILO" => {
                    if self.landed_on_fort_silo {
                        task.completed = true;
                        task_was_completed = true;
                        tasks_to_add.push("SPEAK TO THE GRAND COMMANDER");
                    }
                }
                "SPEAK TO THE GRAND COMMANDER" => {
                    if self.grand_commander_spoken_to {
                        task.completed = true;
                        task_was_completed = true;
                        tasks_to_add.push("LAND THE FIGHTERJET ON THE RACETRACK");
                    }
                }
                "LAND THE FIGHTERJET ON THE RACETRACK" => {
                    if self.racer_returned_to_racetrack {
                        task.completed = true;
                        task_was_completed = true;
                        tasks_to_add.push("PROCEED TO THE STARTING LINE");
                    }
                }
                _ => {}
            }

            if was_complete && task.completed {
                completed_this_frame += 1;
            }
        }
        // Add new tasks outside the iterator loop to avoid borrowing conflicts
        for task_desc in tasks_to_add {
            if !self.has_task(task_desc) {
                self.add_task(task_desc);
            }
        }
        if task_was_completed {
            self.open = true;
			self.auto_close_timer = 7.0;
        }
        completed_this_frame
    }
	
    pub fn update_timer(&mut self, dt: f64) {
        if self.open {
            self.auto_close_timer -= dt;
            if self.auto_close_timer <= 0.0 {
                self.open = false;
            }
        }
    }	

    pub fn draw(&self, c: Context, g: &mut G2d, glyphs: &mut Glyphs) {
        if !self.active || !self.open {
            return;
        }

        let screen_width = crate::config::resolution::WIDTH;
        let font_size = 15;
        let line_height = font_size as f64 + 8.0;
        let start_y = 50.0;
        let start_x = screen_width - 500.0;

        // Calculate backdrop dimensions
        let padding = 8.0; // Padding around the text
        let header_width = glyphs.width(font_size + 2, "TASK:").unwrap_or(0.0);
        let mut max_task_width = header_width;
        for task in self.tasks.iter() {
            let task_text = if task.completed {
                format!("x {}. {}", 0, task.description) // Use 0 as placeholder for calculation
            } else {
                format!("  {}. {}", 0, task.description)
            };
            let current_task_width = glyphs.width(font_size, &task_text).unwrap_or(0.0);
            if current_task_width > max_task_width {
                max_task_width = current_task_width;
            }
        }

        let backdrop_x = start_x - padding;
        let backdrop_y = start_y - (font_size + 2) as f64 - padding; // Account for header font size
        let backdrop_width = max_task_width + (padding * 2.0) + 20.0; // Add extra for numbering/checkbox
        let backdrop_height = (line_height * (self.tasks.len() as f64 + 1.5)) + padding;

        // Draw dark gray backdrop
        rectangle(
            [0.0, 0.0, 0.0, 0.9], // Dark gray with 90% opacity
            [backdrop_x, backdrop_y, backdrop_width, backdrop_height],
            c.transform,
            g,
        );

        text::Text::new_color(LIME_GREEN, font_size + 2)
            .draw(
                "TASK:[T]",
                glyphs,
                &c.draw_state,
                c.transform.trans(start_x, start_y),
                g,
            )
            .ok();

        for (i, task) in self.tasks.iter().enumerate() {
            let y_pos = start_y + line_height * (i as f64 + 1.5);

            let (display_text, color) = if task.completed {
                (format!("x {}. {}", i + 1, task.description), GREY)
            } else {
                (format!("  {}. {}", i + 1, task.description), RED_ORANGE)
            };

            text::Text::new_color(color, font_size)
                .draw(
                    &display_text,
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(start_x, y_pos),
                    g,
                )
                .ok();
        }
    }
}
