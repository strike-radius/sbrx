//File: src/chatbox.rs

use piston_window::*;
use std::collections::VecDeque;
use std::path::PathBuf;

use crate::config::gameplay::WARNING_MESSAGE_DURATION;

/// Enum to define the type of message for standardized color-coding.
#[derive(Clone)]
pub enum MessageType {
    Info,         // White text for info
    Notification, // Red text for system messages or important notifications
    Dialogue,     // Yellow text for character dialogue
    Warning,      // Orange text for on-screen warnings
	Stats, // green text
}

#[derive(PartialEq, Clone, Copy)]
pub enum ChatBoxState {
    Closed,
    TempOpen,
    FullOpen,
}

/// Struct to hold a single message's data, including its content and type.
#[derive(Clone)]
pub struct Message {
    pub text: String,
    pub message_type: MessageType,
}

impl Message {
    /// Returns the appropriate color for a message based on its type.
    pub fn color(&self) -> [f32; 4] {
        match self.message_type {
            MessageType::Info => [1.0, 1.0, 1.0, 1.0],         // white
            MessageType::Notification => [1.0, 1.0, 0.2, 1.0],     // Yellow
            MessageType::Dialogue => [0.0, 1.0, 0.0, 1.0], // Green
            MessageType::Warning => [1.0, 0.5, 0.0, 1.0], // Orange - Not used by new system, but good for completeness
			MessageType::Stats => [0.0, 1.0, 0.0, 1.0], // Green
			// [1.0, 0.2, 0.2, 1.0], // Red
			
        }
    }
}

/// Manages the state and rendering of the in-game chat box.
pub struct ChatBox {
    messages: VecDeque<Message>,
    pub state: ChatBoxState,
    temp_message_count: usize, // Number of lines in the last interaction
    max_visible_messages: usize,
    scroll_offset: usize, // 0 is newest, increases as we scroll up into the history
    enter_key_icon: G2dTexture,
    scroll_up_icon: G2dTexture,
    scroll_down_icon: G2dTexture,
    warnings: VecDeque<(String, f64)>, // Warning text and its remaining lifetime
	pub auto_close_timer: f64,
}

impl ChatBox {
    /// Creates a new ChatBox instance and loads necessary icon assets.
    pub fn new(window: &mut PistonWindow, assets_path: &PathBuf) -> Self {
        let texture_context = &mut window.create_texture_context();
        let settings = TextureSettings::new().filter(Filter::Nearest);

        // Helper closure to load textures, panicking on failure as they are essential.
        let mut load_texture = |name: &str| {
            let path = assets_path.join(format!("{}.png", name));
            Texture::from_path(texture_context, &path, Flip::None, &settings)
                .unwrap_or_else(|e| panic!("Failed to load chatbox texture at {:?}: {}", path, e))
        };

        ChatBox {
            messages: VecDeque::new(),
            state: ChatBoxState::Closed,
            temp_message_count: 0,
            max_visible_messages: 25, // The number of messages that can fit in the box
            scroll_offset: 0,
            enter_key_icon: load_texture("enter_key_icon"),
            scroll_up_icon: load_texture("scroll_up"),
            scroll_down_icon: load_texture("scroll_down"),
            warnings: VecDeque::new(),
			auto_close_timer: 0.0,
        }
    }

    /// Adds a block of messages as a single interaction.
    pub fn add_interaction(&mut self, messages: Vec<(&str, MessageType)>) {
        let mut total_lines_added = 0;
        for (text, message_type) in messages {
            match message_type {
                MessageType::Warning => {
                    self.warnings
                        .push_back((text.to_string(), WARNING_MESSAGE_DURATION));
                }
                _ => {
                    total_lines_added += self.add_message_internal(text, message_type);
                }
            }
        }

        if total_lines_added > 0 {
            self.state = ChatBoxState::TempOpen;
            self.temp_message_count = total_lines_added;
            self.scroll_offset = 0; // Ensure view is at the bottom.
			self.auto_close_timer = 7.0; // Reset auto-close timer on new messages
        }
    }

    // Internal function to add a single message and return the number of lines it created.
    fn add_message_internal(&mut self, text: &str, message_type: MessageType) -> usize {
        let mut lines_added = 0;
        for line in text.lines() {
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() {
                self.messages.push_back(Message {
                    text: trimmed_line.to_string(),
                    message_type: message_type.clone(),
                });
                lines_added += 1;
            }
        }

        // Limit total history to 25 messages to prevent unbounded memory usage.
        while self.messages.len() > 25 {
            self.messages.pop_front();
        }
        lines_added
    }

    /// Updates the lifetime of on-screen warnings.
    pub fn update(&mut self, dt: f64, enter_key_held: bool) {
        if !self.warnings.is_empty() {
            for (_, lifetime) in self.warnings.iter_mut() {
                *lifetime -= dt;
            }
            self.warnings.retain(|(_, lifetime)| *lifetime > 0.0);
        }
		
        if self.state != ChatBoxState::Closed {
            if !enter_key_held {
                self.auto_close_timer -= dt;
                if self.auto_close_timer <= 0.0 {
                    self.state = ChatBoxState::Closed;
                }
            } else {
                // Holding ENTER refreshes the timer to prevent immediate closure upon release
                self.auto_close_timer = 7.0;
            }
        }		
    }

    /// Handles user input for interacting with the chatbox (opening, closing, scrolling).
    pub fn handle_key_press(&mut self, key: Key) {
        match key {
            Key::Return => {
                self.state = match self.state {
                    ChatBoxState::Closed => ChatBoxState::FullOpen,
                    ChatBoxState::TempOpen => ChatBoxState::Closed,
                    ChatBoxState::FullOpen => ChatBoxState::Closed,
                };
                // Reset scroll when closing from full open
                if self.state == ChatBoxState::Closed {
                    self.scroll_offset = 0;
                }
                // Reset timer if opening manually
                if self.state != ChatBoxState::Closed {
                    self.auto_close_timer = 7.0;
                }				
            }
            Key::Up => {
                if self.state == ChatBoxState::FullOpen {
                    self.scroll_up();
					self.auto_close_timer = 7.0; // Refresh timer on interaction
                }
            }
            Key::Down => {
                if self.state == ChatBoxState::FullOpen {
                    self.scroll_down();
					self.auto_close_timer = 7.0; // Refresh timer on interaction
                }
            }
            _ => {}
        }
    }

    /// Clears all messages from the chatbox history and resets the scroll.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll_offset = 0;
        self.temp_message_count = 0;
        self.state = ChatBoxState::Closed;
		self.warnings.clear();
    }

    /// Scrolls up through the message history.
    fn scroll_up(&mut self) {
        let total_messages = self.messages.len();
        if total_messages > self.max_visible_messages {
            let max_offset = total_messages - self.max_visible_messages;
            if self.scroll_offset < max_offset {
                self.scroll_offset += 1;
            }
        }
    }

    /// Scrolls down towards the most recent messages.
    fn scroll_down(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    /// Draws the chatbox and its contents if it is open.
    pub fn draw(&self, c: Context, g: &mut G2d, glyphs: &mut Glyphs) {
        if self.state == ChatBoxState::Closed {
            return;
        }

        let font_size = 15;
        let line_height = font_size as f64 + 5.0;
        let chat_width = 500.0;
        let padding = 10.0;

        let visible_count = match self.state {
            ChatBoxState::TempOpen => self.temp_message_count.min(self.max_visible_messages),
            ChatBoxState::FullOpen => self.messages.len().min(self.max_visible_messages),
            ChatBoxState::Closed => 0,
        };
        let chat_height = (visible_count as f64 * line_height) + (2.0 * padding);

        let screen_height = crate::config::resolution::HEIGHT;
        let box_x = 10.0;
        let box_y = screen_height - chat_height - 10.0;

        // Draw the main chatbox background.
        rectangle(
            [0.1, 0.1, 0.1, 0.9], // Dark semi-transparent background
            [box_x, box_y, chat_width, chat_height],
            c.transform,
            g,
        );

        let total_messages = self.messages.len();
        if total_messages == 0 {
            return;
        }

        // Determine the slice of messages to render based on scroll offset.
        let end_index = total_messages - self.scroll_offset;
        let start_index = end_index.saturating_sub(visible_count);

        // Draw messages from the bottom of the box upwards.
        // OPTIMIZATION: Iterate directly over the VecDeque range without allocating a temporary Vec
        for (i, message) in self.messages.range(start_index..end_index).rev().enumerate() {
            let text_x = box_x + padding;
            // Position each line upwards from the bottom edge of the chatbox.
            let text_y =
                box_y + chat_height - padding - (i as f64 * line_height) - (font_size as f64 / 2.0);

            text::Text::new_color(message.color(), font_size)
                .draw(
                    &message.text,
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(text_x, text_y),
                    g,
                )
                .unwrap_or_else(|e| eprintln!("Failed to draw chat text: {}", e));
        }

        // Draw UI icons next to the chatbox.
        let icon_x = box_x + chat_width + padding;

        let enter_y = box_y + chat_height - self.enter_key_icon.get_height() as f64;
        image(&self.enter_key_icon, c.transform.trans(icon_x, enter_y), g);

        // Display scroll indicators only when there is overflow.
        if self.state == ChatBoxState::FullOpen && total_messages > self.max_visible_messages {
            // Show scroll up icon if there are older messages to see.
            if self.scroll_offset < total_messages - self.max_visible_messages {
                image(&self.scroll_up_icon, c.transform.trans(icon_x, box_y), g);
            }
            // Show scroll down icon if not at the very bottom.
            if self.scroll_offset > 0 {
                image(
                    &self.scroll_down_icon,
                    c.transform.trans(icon_x, box_y + 30.0),
                    g,
                );
            }
        }
    }
    /// Draws any active on-screen warnings.
    pub fn draw_warnings(&self, c: Context, g: &mut G2d, glyphs: &mut Glyphs) {
        if let Some((text, _lifetime)) = self.warnings.front() {
            let font_size = 20;
            let orange = [1.0, 0.5, 0.0, 1.0];

            let text_width = match glyphs.width(font_size, text) {
                Ok(w) => w,
                Err(_) => text.len() as f64 * (font_size as f64 * 0.6), // Fallback
            };

            let screen_width = crate::config::resolution::WIDTH;
            let screen_height = crate::config::resolution::HEIGHT;

            let padding = 15.0;
            let bg_width = text_width + (padding * 2.0);
            let bg_height = font_size as f64 + (padding * 2.0);

            let bg_x = (screen_width - bg_width) / 2.0;
            let bg_y = screen_height / 4.0; // Position from top

            let text_x = bg_x + padding;
            let text_y_baseline = bg_y + padding + font_size as f64;

            let bg_color = [0.0, 0.0, 0.0, 0.5]; // Black with 50% opacity

            rectangle(bg_color, [bg_x, bg_y, bg_width, bg_height], c.transform, g);

            text::Text::new_color(orange, font_size)
                .draw(
                    text,
                    glyphs,
                    &c.draw_state,
                    c.transform.trans(text_x, text_y_baseline),
                    g,
                )
                .unwrap_or_else(|e| eprintln!("Failed to draw warning text: {}", e));
        }
    }
}
