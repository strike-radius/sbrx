// graphics/seven_segment.rs

use piston_window::*;

pub struct SevenSegmentDisplay {
    pub segment_width: f64,
    pub segment_height: f64,
    pub spacing: f64,
}

impl SevenSegmentDisplay {
    pub fn new(segment_width: f64, segment_height: f64, spacing: f64) -> Self {
        SevenSegmentDisplay {
            segment_width,
            segment_height,
            spacing,
        }
    }

    pub fn get_segments_for_digit(&self, digit: u32) -> [bool; 7] {
        match digit {
            0 => [true, true, true, false, true, true, true], // 0
            1 => [false, false, true, false, false, true, false], // 1
            2 => [true, false, true, true, true, false, true], // 2
            3 => [true, false, true, true, false, true, true], // 3
            4 => [false, true, true, true, false, true, false], // 4
            5 => [true, true, false, true, false, true, true], // 5
            6 => [true, true, false, true, true, true, true], // 6
            7 => [true, false, true, false, false, true, false], // 7
            8 => [true, true, true, true, true, true, true],  // 8
            9 => [true, true, true, true, false, true, true], // 9
            _ => [false, false, false, false, false, false, false],
        }
    }

    pub fn draw_segment(
        &self,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        color: [f32; 4],
        context: Context,
        g: &mut G2d,
    ) {
        rectangle(color, [x, y, width, height], context.transform, g);
    }

    pub fn draw_digit(
        &self,
        digit: u32,
        x: f64,
        y: f64,
        color: [f32; 4],
        context: Context,
        g: &mut G2d,
    ) {
        let segments = self.get_segments_for_digit(digit);
        let w = self.segment_width;
        let h = self.segment_height;

        // Segment dimensions
        let horizontal_width = w * 0.8;
        let horizontal_height = h * 0.1;
        let vertical_width = w * 0.1;
        let vertical_height = h * 0.4;

        // Horizontal segment positions (top, middle, bottom)
        let h_x = x + (w - horizontal_width) / 2.0;
        let h_top_y = y;
        let h_mid_y = y + (h - horizontal_height) / 2.0;
        let h_bot_y = y + h - horizontal_height;

        // Vertical segment positions (top-left, top-right, bottom-left, bottom-right)
        let v_left_x = x;
        let v_right_x = x + w - vertical_width;
        let v_top_y = y + horizontal_height;
        let v_bot_y = y + h / 2.0;

        // Draw horizontal segments
        if segments[0] {
            // Top
            self.draw_segment(
                h_x,
                h_top_y,
                horizontal_width,
                horizontal_height,
                color,
                context,
                g,
            );
        }
        if segments[3] {
            // Middle
            self.draw_segment(
                h_x,
                h_mid_y,
                horizontal_width,
                horizontal_height,
                color,
                context,
                g,
            );
        }
        if segments[6] {
            // Bottom
            self.draw_segment(
                h_x,
                h_bot_y,
                horizontal_width,
                horizontal_height,
                color,
                context,
                g,
            );
        }

        // Draw vertical segments
        if segments[1] {
            // Top Left
            self.draw_segment(
                v_left_x,
                v_top_y,
                vertical_width,
                vertical_height,
                color,
                context,
                g,
            );
        }
        if segments[2] {
            // Top Right
            self.draw_segment(
                v_right_x,
                v_top_y,
                vertical_width,
                vertical_height,
                color,
                context,
                g,
            );
        }
        if segments[4] {
            // Bottom Left
            self.draw_segment(
                v_left_x,
                v_bot_y,
                vertical_width,
                vertical_height,
                color,
                context,
                g,
            );
        }
        if segments[5] {
            // Bottom Right
            self.draw_segment(
                v_right_x,
                v_bot_y,
                vertical_width,
                vertical_height,
                color,
                context,
                g,
            );
        }
    }
}
