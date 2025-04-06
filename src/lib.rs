//! # Egui Memory Editor
//!
//! Provides a memory editor to be used with `egui`.
//! Primarily intended for emulation development.
//!
//! Look at [`MemoryEditor`] to get started.
use std::collections::BTreeMap;
use std::ops::Range;

use egui::{Context, Label, Margin, RichText, ScrollArea, Sense, TextEdit, TextWrapMode, Ui, Vec2, Widget, Window};

use crate::option_data::{BetweenFrameData, MemoryEditorOptions};

pub mod option_data;
mod option_ui;
mod utilities;

/// A memory address that should be read from/written to.
pub type Address = usize;

/// The main struct for the editor window.
/// This should persist between frames as it keeps track of quite a bit of state.
#[derive(Clone)]
pub struct MemoryEditor {
    /// The name of the `egui` window, can be left blank.
    window_name: String,
    /// The collection of address ranges, the GUI will start at the lower bound and go up to the upper bound.
    ///
    /// Note this *currently* only supports ranges that have a max of `2^(24+log_2(column_count))` due to `ScrollArea` limitations.
    address_ranges: BTreeMap<String, Range<Address>>,
    /// A collection of options relevant for the `MemoryEditor` window.
    /// Can optionally be serialized/deserialized with `serde`
    pub options: MemoryEditorOptions,
    /// Data for layout between frames, rather hacky.
    frame_data: BetweenFrameData,
    /// The visible range of addresses from the last frame.
    visible_range: Range<Address>,
}

impl MemoryEditor {
    /// Create the MemoryEditor, which should be kept in memory between frames.
    ///
    /// The `read_function` should return one `u8` value from the object which you provide in
    /// either the [`Self::window_ui`] or the [`Self::draw_editor_contents`] method.
    ///
    /// ```no_run
    /// # use egui_memory_editor::MemoryEditor;
    /// # let ctx = egui::Context::default();
    /// let mut memory_base = vec![0xFF; 0xFF];
    /// let mut is_open = true;
    /// let mut memory_editor = MemoryEditor::new().with_address_range("Memory", 0..0xFF);
    ///
    /// // Show a read-only window
    /// memory_editor.window_ui_read_only(&ctx, &mut is_open, &mut memory_base, |mem, addr| mem[addr].into());
    /// ```
    pub fn new() -> Self {
        MemoryEditor {
            window_name: "Memory Editor".to_string(),
            address_ranges: BTreeMap::new(),
            options: Default::default(),
            frame_data: Default::default(),
            visible_range: Default::default(),
        }
    }

    /// Returns the visible range of the last frame.
    ///
    /// Can be useful for asynchronous memory querying.
    pub fn visible_range(&self) -> &Range<Address> {
        &self.visible_range
    }

    /// Create a read-only window and render the memory editor contents within.
    ///
    /// If you want to make your own window/container to be used for the editor contents, you can use [`Self::draw_editor_contents`].
    /// If you wish to be able to write to the memory, you can use [`Self::window_ui`].
    ///
    /// # Arguments
    ///
    /// * `ctx` - The `egui` context.
    /// * `mem` - The memory from which to read.
    /// * `read_fn` - Any closure which takes in a reference to the memory and an address and returns a `u8` value. It can
    /// return `None` if the data at the specified address is not available for whatever reason. This will then be rendered
    /// as `--` (See [`MemoryEditorOptions::none_display_value`])
    pub fn window_ui_read_only<T: ?Sized>(
        &mut self,
        ctx: &Context,
        is_open: &mut bool,
        mem: &mut T,
        read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
    ) {
        // This needs to exist due to the fact we want to use generics, and `Option` needs to know the size of its contents.
        type DummyWriteFunction<T> = fn(&mut T, Address, u8);

        self.window_ui_impl(ctx, is_open, mem, read_fn, None::<DummyWriteFunction<T>>);
    }

    /// Create a window and render the memory editor contents within.
    ///
    /// If you want to make your own window/container to be used for the editor contents, you can use [`Self::draw_editor_contents`].
    /// If you wish for read-only access to the memory, you can use [`Self::window_ui_read_only`].
    ///
    /// # Arguments
    ///
    /// * `ctx` - The `egui` context.
    /// * `mem` - The memory from which to read.
    /// * `read_fn` - Any closure which takes in a reference to the memory and an address and returns a `u8` value. It can
    /// return `None` if the data at the specified address is not available for whatever reason. This will then be rendered
    /// as `--` (See [`MemoryEditorOptions::none_display_value`])
    /// * `write_fn` - Any closure which can take a reference to the memory, an address, and the value to write.
    pub fn window_ui<T: ?Sized>(
        &mut self,
        ctx: &Context,
        is_open: &mut bool,
        mem: &mut T,
        read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
        write_fn: impl FnMut(&mut T, Address, u8),
    ) {
        self.window_ui_impl(ctx, is_open, mem, read_fn, Some(write_fn));
    }

    fn window_ui_impl<T: ?Sized>(
        &mut self,
        ctx: &Context,
        is_open: &mut bool,
        mem: &mut T,
        read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
        write_fn: Option<impl FnMut(&mut T, Address, u8)>,
    ) {
        Window::new(self.window_name.clone())
            .open(is_open)
            .hscroll(false)
            .vscroll(false)
            .resizable(true)
            .show(ctx, |ui| {
                self.shrink_window_ui(ui);
                self.draw_editor_contents_impl(ui, mem, read_fn, write_fn);
            });
    }

    /// Draws the actual memory viewer/editor.
    ///
    /// Can be included in whatever container you want.
    ///
    /// Use [`Self::window_ui`] if you want to have a window with the contents instead.
    ///
    /// This is the read-only variant. See [`Self::draw_editor_contents`] for the read-write variant.
    pub fn draw_editor_contents_read_only<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
    ) {
        // This needs to exist due to the fact we want to use generics, and `Option` needs to know the size of its contents.
        type DummyWriteFunction<T> = fn(&mut T, Address, u8);

        self.draw_editor_contents_impl(ui, mem, read_fn, None::<DummyWriteFunction<T>>);
    }

    /// Draws the actual memory viewer/editor.
    ///
    /// Can be included in whatever container you want.
    ///
    /// Use [`Self::window_ui`] if you want to have a window with the contents instead.
    ///
    /// If the read-only variant is preferred see [`Self::draw_editor_contents_read_only`].
    pub fn draw_editor_contents<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
        write_fn: impl FnMut(&mut T, Address, u8),
    ) {
        self.draw_editor_contents_impl(ui, mem, read_fn, Some(write_fn));
    }

    fn draw_editor_contents_impl<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        mut read_fn: impl FnMut(&mut T, Address) -> Option<u8>,
        mut write_fn: Option<impl FnMut(&mut T, Address, u8)>,
    ) {
        assert!(
            !self.address_ranges.is_empty(),
            "At least one address range needs to be added to render the contents!"
        );

        self.draw_options_area(ui, mem, &mut read_fn);

        ui.separator();

        let MemoryEditorOptions {
            show_ascii,
            column_count,
            address_text_colour,
            highlight_text_colour,
            selected_address_range,
            memory_editor_address_text_style,
            ..
        } = self.options.clone();

        let line_height = self.get_line_height(ui);
        let address_space = self.address_ranges.get(&selected_address_range).unwrap().clone();
        // This is janky, but can't think of a better way.
        let address_characters = format!("{:X}", address_space.end - 1).chars().count();
        let max_lines = (address_space.len() + column_count - 1) / column_count; // div_ceil

        // For when we're editing memory, don't use the `Response` object as that would screw over downward scrolling.
        self.handle_keyboard_edit_input(&address_space, ui.ctx());

        let mut scroll = ScrollArea::vertical()
            .id_salt(selected_address_range)
            .max_height(f32::INFINITY)
            .auto_shrink([false, true]);

        // Scroll to the goto area address line.
        if let Some(line) = self.frame_data.goto_address_line.take() {
            let new_offset = (line_height + ui.spacing().item_spacing.y) * (line as f32);
            scroll = scroll.vertical_scroll_offset(new_offset);
        }

        scroll.show_rows(ui, line_height, max_lines, |ui, line_range| {
            // Persist the visible range for future queries.
            let start_address_range = address_space.start + (line_range.start * column_count);
            let end_address_range = address_space.start + (line_range.end * column_count);
            self.visible_range = start_address_range..end_address_range;

            egui::Grid::new("mem_edit_grid")
                .striped(true)
                .spacing(Vec2::new(15.0, ui.style().spacing.item_spacing.y))
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.style_mut().spacing.item_spacing.x = 3.0;

                    for start_row in line_range.clone() {
                        let start_address = address_space.start + (start_row * column_count);
                        let line_range = start_address..start_address + column_count;
                        let highlight_in_range = matches!(self.frame_data.selected_highlight_address, Some(address) if line_range.contains(&address));

                        let start_text = RichText::new(format!("0x{:01$X}:", start_address, address_characters))
                            .color(if highlight_in_range { highlight_text_colour } else { address_text_colour })
                            .text_style(memory_editor_address_text_style.clone());

                        ui.label(start_text);

                        self.draw_memory_values(ui, mem, &mut read_fn, &mut write_fn, start_address, &address_space);

                        if show_ascii {
                            self.draw_ascii_sidebar(ui, mem, &mut read_fn, start_address, &address_space);
                        }

                        ui.end_row();
                    }
                });
            // After we've drawn the area we want to resize to we want to save this size for the next frame.
            // In case it has become smaller we'll shrink the window.
            self.frame_data.previous_frame_editor_width = ui.min_rect().width();
        });
    }

    fn draw_memory_values<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        read_fn: &mut impl FnMut(&mut T, Address) -> Option<u8>,
        write_fn: &mut Option<impl FnMut(&mut T, Address, u8)>,
        start_address: Address,
        address_space: &Range<Address>,
    ) {
        let frame_data = &mut self.frame_data;
        let options = &self.options;
        let mut read_only = frame_data.selected_edit_address.is_none() || write_fn.is_none();

        // div_ceil
        for grid_column in 0..(options.column_count + 7) / 8 {
            let start_address = start_address + 8 * grid_column;

            // Each grid column is 8 bytes, where each byte is one 'sub-column'.
            ui.horizontal(|ui| {
                let column_count = (options.column_count - 8 * grid_column).min(8);

                for column_index in 0..column_count {
                    let memory_address = start_address + column_index;

                    if !address_space.contains(&memory_address) {
                        break;
                    }

                    let mem_val: Option<u8> = read_fn(mem, memory_address);
                    // If the read function can't read for whatever reason we'll just assume some temporary `--` value.
                    let label_text = match mem_val {
                        Some(val) => format!("{:02X}", val),
                        None => options.none_display_value.clone(),
                    };

                    // Memory Value Labels
                    if !read_only
                        && matches!(frame_data.selected_edit_address, Some(address) if address == memory_address)
                    {
                        // For Editing
                        let response = ui.add(
                            TextEdit::singleline(&mut frame_data.selected_edit_address_string)
                                .desired_width(frame_data.previous_frame_text_edit_size)
                                .margin(Margin::symmetric(0, 0))
                                .font(options.memory_editor_text_style.clone())
                                .hint_text(label_text)
                                .id_source(frame_data.selected_edit_address),
                        );

                        if frame_data.selected_edit_address_request_focus {
                            frame_data.selected_edit_address_request_focus = false;
                            response.request_focus();
                        }

                        // Filter out any non Hex-Digit, there doesn't seem to be a method in TextEdit for this.
                        frame_data
                            .selected_edit_address_string
                            .retain(|c| c.is_ascii_hexdigit());

                        // Don't want more than 2 digits
                        if frame_data.selected_edit_address_string.chars().count() >= 2 {
                            let next_address = memory_address + 1;
                            let new_value = u8::from_str_radix(&frame_data.selected_edit_address_string[0..2], 16);

                            if let Ok(value) = new_value {
                                if let Some(write_fns) = write_fn.as_mut() {
                                    write_fns(mem, memory_address, value);
                                }
                            }

                            frame_data.set_selected_edit_address(Some(next_address), address_space);
                        } else if !response.has_focus() {
                            // We use has_focus() instead of response.inner.lost_focus() due to the latter
                            // having a bug where it doesn't detect if it lost focus when you scroll.
                            frame_data.set_selected_edit_address(None, address_space);
                            read_only = true;
                        }
                    } else {
                        // Read-only values.
                        let mut text = RichText::new(label_text).text_style(options.memory_editor_text_style.clone());

                        if options.show_zero_colour && (matches!(mem_val, Some(val) if val == 0) || mem_val.is_none()) {
                            text = text.color(options.zero_colour);
                        } else {
                            text = text.color(ui.style().visuals.text_color());
                        };

                        if frame_data.should_highlight(memory_address) {
                            text = text.color(options.highlight_text_colour);
                        }

                        if frame_data.should_subtle_highlight(memory_address, options.data_preview.selected_data_format)
                        {
                            text = text.background_color(ui.style().visuals.code_bg_color);
                        }

                        let response = Label::new(text).sense(Sense::click()).ui(ui);
                        // For use with the `Edit` widget, keep track of the size of ordinary display to keep column jitter at bay
                        frame_data.previous_frame_text_edit_size = response.rect.width();

                        // Right click always selects.
                        if response.secondary_clicked() {
                            frame_data.set_highlight_address(memory_address);
                        }

                        // Left click depends on read only mode.
                        if response.clicked() {
                            if write_fn.is_some() {
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

    fn draw_ascii_sidebar<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        read_fn: &mut impl FnMut(&mut T, Address) -> Option<u8>,
        start_address: Address,
        address_space: &Range<Address>,
    ) {
        let options = &self.options;

        ui.horizontal(|ui| {
            ui.add(egui::Separator::default().vertical().spacing(3.0));
            ui.style_mut().spacing.item_spacing.x = 0.0;

            ui.horizontal(|ui| {
                for i in 0..options.column_count {
                    let memory_address = start_address + i;

                    if !address_space.contains(&memory_address) {
                        break;
                    }

                    let mem_val: u8 = read_fn(mem, memory_address).unwrap_or(0);
                    // Check if it's a printable ASCII character
                    let character = if !(32..128).contains(&mem_val) {
                        '.'
                    } else {
                        mem_val as char
                    };
                    let mut text = RichText::new(character).text_style(options.memory_editor_ascii_text_style.clone());

                    if self.frame_data.should_highlight(memory_address) {
                        text = text
                            .color(self.options.highlight_text_colour)
                            .background_color(ui.style().visuals.code_bg_color);
                    }

                    ui.label(text);
                }
            });
        });
    }

    /// Return the line height for the current provided `Ui` and selected `TextStyle`s
    fn get_line_height(&self, ui: &mut Ui) -> f32 {
        let address_size = ui.text_style_height(&self.options.memory_editor_address_text_style);
        let body_size = ui.text_style_height(&self.options.memory_editor_text_style);
        let ascii_size = ui.text_style_height(&self.options.memory_editor_ascii_text_style);
        address_size.max(body_size).max(ascii_size)
    }

    /// Shrink the window to the previous frame's memory viewer's width.
    /// This essentially allows us to only have height resize, and have width grow/shrink as appropriate.
    fn shrink_window_ui(&self, ui: &mut Ui) {
        // This should take the `min` of ui.min_rect().width() and the frame data width, but that seems to have issues at the moment.
        ui.set_max_width(self.frame_data.previous_frame_editor_width);
    }

    /// Check for arrow keys when we're editing a memory value at an address.
    fn handle_keyboard_edit_input(&mut self, address_range: &Range<Address>, ctx: &Context) {
        use egui::Key::*;
        const KEYS: [egui::Key; 4] = [ArrowLeft, ArrowRight, ArrowDown, ArrowUp];

        let Some(current_address) = self.frame_data.selected_edit_address else {
            return;
        };

        let key_pressed = KEYS.iter().find(|&&k| ctx.input(|i| i.key_pressed(k)));
        if let Some(key) = key_pressed {
            let next_address = match key {
                ArrowDown => current_address + self.options.column_count,
                ArrowLeft => current_address.saturating_sub(1),
                ArrowRight => current_address.saturating_add(1),
                ArrowUp => {
                    if current_address < self.options.column_count {
                        0
                    } else {
                        current_address - self.options.column_count
                    }
                }
                _ => unreachable!(),
            };

            self.frame_data
                .set_selected_edit_address(Some(next_address), address_range);
        }
    }

    // ** Builder methods **

    /// Set the window title, only relevant if using the `window_ui()` call.
    #[must_use]
    pub fn with_window_title(mut self, title: impl Into<String>) -> Self {
        self.window_name = title.into();
        self
    }

    /// Add an address range to the range list.
    /// Multiple address ranges can be added, and will be displayed in the UI by a drop-down box if more than one
    /// range was added.
    ///
    /// The first range that is added will be displayed by default when launching the UI.
    ///
    /// The UI will query your set `read_function` with the values within this `Range`
    #[inline]
    #[must_use]
    pub fn with_address_range(mut self, range_name: impl Into<String>, address_range: Range<Address>) -> Self {
        self.set_address_range(range_name, address_range);
        self
    }

    /// Add or update an address range.
    ///
    /// See also [`Self::with_address_range`]
    pub fn set_address_range(&mut self, range_name: impl Into<String>, address_range: Range<Address>) {
        self.address_ranges.insert(range_name.into(), address_range);
        self.frame_data.memory_range_combo_box_enabled = self.address_ranges.len() > 1;

        // Only update the current selected range if nothing else has been selected to prevent annoying jitter.
        if self.options.selected_address_range.is_empty() {
            if let Some((name, _)) = self.address_ranges.iter().next() {
                self.options.selected_address_range = name.clone();
            }
        }
    }

    /// Set the memory options, useful if you use the `persistence` feature.
    #[inline]
    #[must_use]
    pub fn with_options(mut self, options: MemoryEditorOptions) -> Self {
        self.set_options(options);
        self
    }

    /// Set the memory options, useful if you use the `persistence` feature.
    ///
    /// See also [`Self::with_options`]
    pub fn set_options(&mut self, options: MemoryEditorOptions) {
        self.options = options;
    }
}

impl Default for MemoryEditor {
    fn default() -> Self {
        MemoryEditor::new()
    }
}
