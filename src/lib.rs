use std::collections::BTreeMap;
use std::ops::Range;

use egui::{Align, CtxRef, Label, Layout, TextEdit, Ui, Vec2, Window};

use crate::option_data::{BetweenFrameData, MemoryEditorOptions};

mod list_clipper;
pub mod option_data;
mod option_ui;
mod utilities;

/// Reads a value present at the provided address in the object `T`.
///
/// # Arguments
///
/// - `&mut T`: the object on which the read should be performed.
/// - `usize`: The address of the read.
pub type ReadFunction<T> = fn(&mut T, usize) -> u8;
/// Writes the changes the user made to the `T` object.
///
/// # Arguments
///
/// - `&mut T`: the object whose state is to be updated.
/// - `usize`: The address of the intended write.
/// - `u8`: The value set by the user for the provided address.
pub type WriteFunction<T> = fn(&mut T, usize, u8);

/// The main struct for the editor window.
/// This should persist between frames as it keeps track of quite a bit of state.
#[derive(Clone)]
pub struct MemoryEditor<T> {
    /// The name of the `egui` window, can be left blank.
    window_name: String,
    /// The function used for getting the values out of the provided type `T` and displaying them.
    read_function: ReadFunction<T>,
    /// The function used when attempts are made to change values within the GUI.
    write_function: Option<WriteFunction<T>>,
    /// The collection of address ranges, the GUI will start at the lower bound and go up to the upper bound.
    ///
    /// Note this *currently* only supports ranges that have a max of `2^(24+log_2(column_count))` due to `ScrollArea` limitations.
    address_ranges: BTreeMap<String, Range<usize>>,
    /// A collection of options relevant for the `MemoryEditor` window.
    /// Can optionally be serialized/deserialized with `serde`
    pub options: MemoryEditorOptions,
    /// Data for layout between frames, rather hacky.
    frame_data: BetweenFrameData,
}

impl<T> MemoryEditor<T> {
    /// Create the MemoryEditor, which should be kept in memory between frames.
    ///
    /// The `read_function` should return one `u8` value from the object which you provide in
    /// either the [`Self::window_ui`] or the [`Self::draw_editor_contents`] method.
    ///
    /// ```
    /// # use egui_memory_editor::MemoryEditor;
    /// let mut memory_base = vec![0xFF; 0xFF];
    /// let mut memory_editor: MemoryEditor<Vec<u8>> = MemoryEditor::new(|memory, address| memory[address]);
    /// ```
    pub fn new(read_function: ReadFunction<T>) -> Self {
        MemoryEditor {
            window_name: "Memory Editor".to_string(),
            read_function,
            write_function: None,
            address_ranges: BTreeMap::new(),
            options: Default::default(),
            frame_data: Default::default(),
        }
    }

    /// Create a window and render the memory editor contents within.
    ///
    /// If you want to make your own window/container to be used for the editor contents, you can use [`Self::draw_editor_contents`].
    pub fn window_ui(&mut self, ctx: &CtxRef, memory: &mut T) {
        let mut is_open = self.options.is_open;

        Window::new(self.window_name.clone())
            .open(&mut is_open)
            .scroll(false)
            .resizable(true)
            .show(ctx, |ui| {
                self.shrink_window_ui(ui);
                self.draw_editor_contents(ui, memory);
            });

        self.options.is_open = is_open;
    }

    /// Draws the actual memory viewer/editor.
    ///
    /// Can be included in whatever container you want.
    ///
    /// Use [`Self::window_ui`] if you want to have a window with the contents instead.
    pub fn draw_editor_contents(&mut self, ui: &mut Ui, memory: &mut T) {
        assert!(self.address_ranges.len() > 0, "At least one address range needs to be added to render the contents!");

        self.draw_options_area(ui, memory);

        ui.separator();

        let MemoryEditorOptions {
            show_ascii_sidebar,
            column_count,
            address_text_colour,
            selected_address_range,
            memory_editor_address_text_style,
            ..
        } = self.options.clone();

        let line_height = self.get_line_height(ui);
        let address_space = self.address_ranges.get(&selected_address_range).unwrap().clone();
        // This is janky, but can't think of a better way.
        let address_characters = format!("{:X}", address_space.end).chars().count();
        // Memory Editor Part.
        let max_lines = (address_space.len() + column_count - 1) / column_count; // div_ceil

        // For when we're editing memory, don't use the `Response` object as that would screw over downward scrolling.
        self.handle_keyboard_edit_input(&address_space, ui.ctx());

        list_clipper::ClippedScrollArea::auto_sized(max_lines, line_height)
            .with_start_line(std::mem::take(&mut self.frame_data.goto_address_line))
            .show(ui, |ui, line_range| {
                egui::Grid::new("mem_edit_grid")
                    .striped(true)
                    .spacing(Vec2::new(15.0, ui.style().spacing.item_spacing.y))
                    .show(ui, |ui| {
                        ui.style_mut().wrap = Some(false);
                        ui.style_mut().spacing.item_spacing.x = 3.0;

                        for start_row in line_range.clone() {
                            let start_address = address_space.start + (start_row * column_count);
                            ui.add(Label::new(format!("0x{:01$X}:", start_address, address_characters))
                                .text_color(address_text_colour)
                                .text_style(memory_editor_address_text_style));

                            self.draw_memory_values(ui, memory, start_address, &address_space);

                            if show_ascii_sidebar {
                                self.draw_ascii_sidebar(ui, memory, start_address, &address_space);
                            }

                            ui.end_row();
                        }
                    });
                // After we've drawn the area we want to resize to we want to save this size for the next frame.
                // In case it has became smaller we'll shrink the window.
                self.frame_data.previous_frame_editor_width = ui.min_rect().width();
            });
    }

    fn draw_memory_values(&mut self, ui: &mut Ui, memory: &mut T, start_address: usize, address_space: &Range<usize>) {
        let frame_data = &mut self.frame_data;
        let options = &self.options;
        let read_function = self.read_function;
        let write_function = &self.write_function;
        let mut read_only = frame_data.selected_edit_address.is_none() || write_function.is_none();

        for grid_column in 0..(options.column_count + 7) / 8 { // div_ceil
            let start_address = start_address + 8 * grid_column;
            // We use columns here instead of horizontal_for_text() to keep consistent spacing for non-monospace fonts.
            // When fonts are more customizable (e.g, we can accept a `Font` as a setting instead of `TextStyle`) I'd like
            // to switch to horizontal_for_text() as we can then just assume a decent Monospace font provided by the user.
            ui.columns((options.column_count - 8 * grid_column).min(8), |columns| {
                for (i, column) in columns.iter_mut().enumerate() {
                    let memory_address = start_address + i;

                    if !address_space.contains(&memory_address) {
                        break;
                    }

                    let mem_val: u8 = read_function(memory, memory_address);

                    let label_text = format!("{:02X}", mem_val);

                    // Memory Value Labels
                    if !read_only && matches!(frame_data.selected_edit_address, Some(address) if address == memory_address) {
                        // For Editing
                        let response = column.with_layout(Layout::right_to_left(), |ui| {
                            ui.add(TextEdit::singleline(&mut frame_data.selected_edit_address_string)
                                .text_style(options.memory_editor_text_style)
                                .hint_text(label_text))
                        });
                        if frame_data.selected_edit_address_request_focus {
                            frame_data.selected_edit_address_request_focus = false;
                            column.memory().request_kb_focus(response.inner.id);
                        }

                        // Filter out any non Hex-Digit, there doesn't seem to be a method in TextEdit for this.
                        frame_data.selected_edit_address_string.retain(|c| c.is_ascii_hexdigit());

                        // Don't want more than 2 digits
                        if frame_data.selected_edit_address_string.chars().count() >= 2 {
                            let next_address = memory_address + 1;
                            let new_value = u8::from_str_radix(&frame_data.selected_edit_address_string[0..2], 16);

                            if let Ok(value) = new_value {
                                write_function.unwrap()(memory, memory_address, value);
                            }

                            frame_data.set_selected_edit_address(Some(next_address), address_space);
                        } else if !column.ctx().memory().has_kb_focus(response.inner.id) {
                            // We use has_kb_focus() instead of response.inner.lost_kb_focus() due to the latter
                            // having a bug where it doesn't detect it lost focus when you scroll.
                            frame_data.set_selected_edit_address(None, address_space);
                            read_only = true;
                        }
                    } else {
                        // Read-only values.
                        let mut label = Label::new(label_text)
                            .text_style(options.memory_editor_text_style);

                        label = if options.show_zero_colour && mem_val == 0 {
                            label.text_color(options.zero_colour)
                        } else {
                            label.text_color(column.style().visuals.text_color())
                        };

                        if frame_data.should_highlight(memory_address) {
                            label = label.text_color(options.highlight_colour);
                        }

                        if frame_data.should_subtle_highlight( memory_address, options.data_preview_options.selected_data_format) {
                            label = label.background_color(column.style().visuals.code_bg_color);
                        }

                        // This particular layout is necessary to stop the memory values gradually shifting over to the right
                        // Presumably due to some floating point error when using left_to_right()
                        let response = column.with_layout(Layout::right_to_left(), |ui| ui.add(label));
                        // Right click always selects.
                        if response.inner.secondary_clicked() {
                            frame_data.set_highlight_address(memory_address);
                        }
                        // Left click depends on read only mode.
                        if response.inner.clicked() {
                            if write_function.is_some() {
                                frame_data.set_selected_edit_address(Some(memory_address), address_space);
                            } else {
                                frame_data.set_highlight_address(memory_address);
                            }
                        }
                    }
                }
            });
        }
    }

    fn draw_ascii_sidebar(&mut self, ui: &mut Ui, memory: &mut T, start_address: usize, address_space: &Range<usize>) {
        let options = &self.options;
        // Not pretty atm, needs a better method: TODO
        ui.horizontal(|ui| {
            ui.add(egui::Separator::new().vertical().spacing(3.0));
            ui.style_mut().spacing.item_spacing.x = 0.0;
            ui.columns(options.column_count, |columns| {
                for (i, column) in columns.iter_mut().enumerate() {
                    let memory_address = start_address + i;

                    if !address_space.contains(&memory_address) {
                        break;
                    }

                    let mem_val: u8 = (self.read_function)(memory, memory_address);
                    let character = if mem_val < 32 || mem_val >= 128 { '.' } else { mem_val as char };
                    let mut label = egui::Label::new(character).text_style(options.memory_editor_ascii_text_style);

                    if self.frame_data.should_highlight(memory_address) {
                        label = label.text_color(self.options.highlight_colour)
                            .background_color(column.style().visuals.code_bg_color);
                    }

                    column.with_layout(Layout::bottom_up(Align::Center), |ui| {
                        ui.add(label);
                    });
                }
            });
        });
    }

    /// Return the line height for the current provided `Ui` and selected `TextStyle`s
    fn get_line_height(&self, ui: &mut Ui) -> f32 {
        let address_size = ui.fonts()[self.options.memory_editor_address_text_style].row_height();
        let body_size = ui.fonts()[self.options.memory_editor_text_style].row_height();
        let ascii_size = ui.fonts()[self.options.memory_editor_ascii_text_style].row_height();
        address_size.max(body_size).max(ascii_size) + ui.style().spacing.item_spacing.y
    }

    /// Shrink the window to the previous frame's memory viewer's width.
    /// This essentially allows us to only have height resize, and have width grow/shrink as appropriate.
    fn shrink_window_ui(&self, ui: &mut Ui) {
        ui.set_max_width(ui.min_rect().width().max(self.frame_data.previous_frame_editor_width));
    }

    /// Check for arrow keys when we're editing a memory value at an address.
    fn handle_keyboard_edit_input(&mut self, address_range: &Range<usize>, ctx: &CtxRef) {
        use egui::Key::*;
        if self.frame_data.selected_edit_address.is_none() {
            return;
        }
        // We know it must exist otherwise this function can't be called, so safe to unwrap.
        let current_address = self.frame_data.selected_edit_address.unwrap();
        let keys = [ArrowLeft, ArrowRight, ArrowDown, ArrowUp];
        let key_pressed = keys.iter().filter(|&&k| ctx.input().key_pressed(k)).next();
        if let Some(key) = key_pressed {
            let next_address = match key {
                ArrowDown => current_address + self.options.column_count,
                ArrowLeft => current_address - 1,
                ArrowRight => current_address + 1,
                ArrowUp => current_address - self.options.column_count,
                _ => unreachable!()
            };

            self.frame_data.set_selected_edit_address(Some(next_address), address_range);
            // Follow the edit cursor whilst moving with the arrow keys.
            //self.frame_data.goto_address_line = Some(next_address / self.options.column_count);
        }
    }

    // ** Builder methods **

    /// Set the window title, only relevant if using the `window_ui()` call.
    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.window_name = title.into();
        self
    }

    /// Set the function used to write to the provided object `T`.
    ///
    /// This will give the UI write capabilities, and will therefore no longer be `read_only`.
    pub fn with_write_function(mut self, write_function: WriteFunction<T>) -> Self {
        self.write_function = Some(write_function);
        self
    }

    /// Add an address range to the range list.
    /// Multiple address ranges can be added, and will be displayed in the UI by a drop-down box if more than one
    /// range was added.
    ///
    /// The first range that is added will be displayed by default when launching the UI.
    ///
    /// The UI will query your set `read_function` with the values within this `Range`
    #[must_use]
    pub fn with_address_range(mut self, range_name: impl Into<String>, address_range: Range<usize>) -> Self {
        self.address_ranges.insert(range_name.into(), address_range);
        self.frame_data.memory_range_combo_box_enabled = self.address_ranges.len() > 1;
        if let Some((name, _)) = self.address_ranges.iter().next() {
            self.options.selected_address_range = name.clone();
        }
        self
    }

    /// Set the memory options, useful if you use the `persistence` feature.
    pub fn with_options(mut self, options: MemoryEditorOptions) -> Self {
        self.options = options;
        self
    }
}