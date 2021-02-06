use std::ops::Range;

use egui::{Ui, Vec2};

use crate::egui_utilities::*;

/// A simple utility to make it easier to only insert/draw `Ui` elements that will actually be visible in the current
///  `ScrollArea`.
pub struct ScrollAreaClipper {
    theoretical_max_height: f64,
    max_lines: usize,
    line_height: f32,
}

impl ScrollAreaClipper {
    pub fn new(max_lines: usize, line_height: f32) -> Self {
        Self {
            theoretical_max_height: max_lines as f64 * line_height as f64,
            max_lines,
            line_height,
        }
    }

    /// Move the cursor of the provided `Ui` to the first line, ready to start drawing.
    ///
    /// Should be used before drawing anything within the `ScrollArea`.
    pub fn begin(&self, ui: &mut Ui) {
        let start = self.display_start_f32(ui);
        ui.allocate_space(Vec2::new(0.0, start.min(self.theoretical_max_height as f32)));
    }

    /// Pad out the remaining space until the `max_lines` to ensure a consistent scroller length.
    ///
    /// Should be used as the last `Ui` function in a `ScrollArea`
    pub fn finish(&self, ui: &mut Ui) {
        let scroll_y = get_current_scroll(ui).0 + ui.clip_rect().max.y;
        // Always leave a little extra white space on the bottom to ensure the last line is visible.
        ui.allocate_space(Vec2::new(0.0, (self.theoretical_max_height as f32 - scroll_y).max(5.0)));
    }

    pub fn display_line_start(&self, ui: &Ui) -> usize {
        (self.display_start_f32(ui) / self.line_height) as usize
    }

    pub fn display_line_end(&self, ui: &Ui) -> usize {
        let start = self.display_line_start(ui);
        let clip_lines = (ui.clip_rect().max.y / self.line_height) as usize;
        (start + clip_lines).min(self.max_lines)
    }

    pub fn get_current_line_range(&self, ui: &Ui) -> Range<usize> {
        (self.display_line_start(ui)..self.display_line_end(ui))
    }

    fn display_start_f32(&self, ui: &Ui) -> f32 {
        let (scroll_y, _) = get_current_scroll(ui);
        scroll_y
    }
}