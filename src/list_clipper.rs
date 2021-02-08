use std::ops::Range;

use egui::{ScrollArea, Ui, Vec2, Align};

use crate::utilities::*;

/// Thin wrapper around a `ScrollArea` to reduce indentation when using the `ScrollAreaClipper`
#[derive(Clone, Debug)]
pub struct ClippedScrollArea {
    scroll_area: ScrollArea,
    clipper: ScrollAreaClipper,
}

impl ClippedScrollArea {
    pub fn auto_sized(max_lines: usize, line_height: f32) -> Self {
        Self::from_max_height(max_lines, line_height, f32::INFINITY)
    }

    pub fn from_max_height(max_lines: usize, line_height: f32, max_height: f32) -> Self {
        Self {
            scroll_area: ScrollArea::from_max_height(max_height),
            clipper: ScrollAreaClipper::new(max_lines, line_height),
        }
    }

    /// Start using the `ClippedScrollArea`.
    ///
    /// The `add_contents` provides a `Ui` object, as well as a non-inclusive `Range<usize>` of the current visible lines.
    pub fn show<R>(self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui, Range<usize>) -> R) -> R {
        let scroll_area = self.scroll_area.clone();
        scroll_area.show(ui, |ui| self.clipper.show(ui, add_contents))
    }

    /// If `false` (default), the scroll bar will be hidden when not needed/
    /// If `true`, the scroll bar will always be displayed even if not needed.
    pub fn always_show_scroll(mut self, always_show_scroll: bool) -> Self {
        self.scroll_area = self.scroll_area.always_show_scroll(always_show_scroll);
        self
    }

    /// A source for the unique `Id`, e.g. `.id_source("second_scroll_area")` or `.id_source(loop_index)`.
    pub fn id_source(mut self, id_source: impl std::hash::Hash) -> Self {
        self.scroll_area = self.scroll_area.id_source(id_source);
        self
    }

    /// Set the vertical scroll offset position.
    ///
    /// See also: [`Ui::scroll_to_cursor`](egui::Ui::scroll_to_cursor) and
    /// [`Response::scroll_to_me`](egui::Response::scroll_to_me)
    pub fn scroll_offset(mut self, offset: f32) -> Self {
        self.scroll_area = self.scroll_area.scroll_offset(offset);
        self
    }

    /// If `start_line` is [`Option::Some`] then the clipper will move to that line instead of the current scroll.
    pub fn with_start_line(mut self, start_line: Option<usize>) -> Self {
        self.clipper = self.clipper.with_start_line(start_line);
        self
    }
}

/// A simple utility to make it easier to only insert/draw `Ui` elements that will actually be visible in the current
///  `ScrollArea` while maintaining the size of said area.
#[derive(Debug, Copy, Clone)]
pub struct ScrollAreaClipper {
    theoretical_max_height: f64,
    max_lines: usize,
    line_height: f32,
    start_line: Option<usize>,
}

impl ScrollAreaClipper {
    pub fn new(max_lines: usize, line_height: f32) -> Self {
        Self {
            theoretical_max_height: max_lines as f64 * line_height as f64,
            max_lines,
            line_height,
            start_line: None
        }
    }

    /// Start using the `ScrollAreaClipper`. Automatically call the relevant `begin()` and `finish` functions.
    ///
    /// The `add_contents` provides a `Ui` object, as well as a non-inclusive `Range<usize>` of the current visible lines.
    pub fn show<R>(mut self, ui: &mut Ui, add_contents: impl FnOnce(&mut Ui, Range<usize>) -> R) -> R {
        self.begin(ui);
        let response = add_contents(ui, self.get_current_line_range(ui));
        self.finish(ui);
        response
    }

    /// Move the cursor of the provided `Ui` to the first line, ready to start drawing.
    ///
    /// Should be used before drawing anything within the `ScrollArea`.
    pub fn begin(&self, ui: &mut Ui) {
        let start = self.display_start_f32(ui);
        ui.allocate_space(Vec2::new(0.0, start.min(self.theoretical_max_height as f32)));
        // Need to manually scroll to the start line
        if self.start_line.is_some() {
            ui.scroll_to_cursor(Align::TOP);
        }
    }

    /// Pad out the remaining space until the `max_lines` to ensure a consistent scroller length.
    ///
    /// Should be used as the last `Ui` function in a `ScrollArea`
    pub fn finish(&mut self, ui: &mut Ui) {
        let scroll_y = get_current_scroll(ui).0 + ui.clip_rect().max.y;
        // We know we'll now have completed our obligation to draw at the start line, so remove it.
        self.start_line = None;
        // Always leave a little extra white space on the bottom to ensure the last line is visible.
        ui.allocate_space(Vec2::new(0.0, (self.theoretical_max_height as f32 - scroll_y).max(5.0)));
    }

    pub fn get_current_line_range(&self, ui: &Ui) -> Range<usize> {
        self.display_line_start(ui)..self.display_line_end(ui)
    }

    /// If `start_line` is [`Option::Some`] then the clipper will move to that line instead of the current scroll.
    pub fn with_start_line(mut self, start_line: Option<usize>) -> Self {
        self.start_line = start_line;
        self
    }

    fn display_line_start(&self, ui: &Ui) -> usize {
        (self.display_start_f32(ui) / self.line_height) as usize
    }

    fn display_line_end(&self, ui: &Ui) -> usize {
        let start = self.display_line_start(ui);
        let clip_lines = (ui.clip_rect().max.y / self.line_height) as usize;
        (start + clip_lines).min(self.max_lines)
    }

    fn display_start_f32(&self, ui: &Ui) -> f32 {
        if let Some(&line) = self.start_line.as_ref() {
            self.line_height * line as f32
        } else {
            let (scroll_y, _) = get_current_scroll(ui);
            scroll_y
        }
    }
}
