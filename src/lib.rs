use std::ops::Range;

use egui::{Align, Color32, CtxRef, FontDefinitions, Label, Layout, Pos2, Rect, TextStyle, Ui, Vec2, Window};
use num::Integer;

use crate::egui_utilities::*;
use crate::option_data::{MemoryEditorOptions, BetweenFrameUiData};

mod egui_utilities;
mod list_clipper;
pub mod option_data;

/// Reads a value present at the provided address in the object `T`.
///
/// # Arguments
///
/// - `&mut T`: the object on which the read should be performed.
/// - `usize`: The address of the read.
pub type ReadFunction<T> = Box<dyn FnMut(&mut T, usize) -> u8>;
/// Writes the changes the user made to the `T` object.
///
/// # Arguments
///
/// - `&mut T`: the object whose state is to be updated.
/// - `usize`: The address of the intended write.
/// - `u8`: The value set by the user for the provided address.
pub type WriteFunction<T> = Box<dyn FnMut(&mut T, usize, u8)>;

pub struct MemoryEditor<T> {
    /// The name of the `egui` window, can be left blank.
    window_name: String,
    /// The function used for getting the values out of the provided type `T` and displaying them.
    read_function: Option<ReadFunction<T>>,
    /// The function used when attempts are made to change values within the GUI.
    write_function: Option<WriteFunction<T>>,
    /// The range of possible values to be displayed, the GUI will start at the lower bound and go up to the upper bound.
    ///
    /// Note this *currently* only supports a range that has a max of `2^24`, due to `ScrollArea` limitations.
    address_space: Range<usize>,
    /// The amount of characters used to indicate the current address in the sidebar of the UI.
    /// Is derived from the provided `address_space.end`
    address_characters: usize,
    /// When `true` will disallow any edits, ensuring the `write_function` will never be called.
    /// The latter therefore doesn't need to be set.
    read_only: bool,
    /// A collection of options relevant for the `MemoryEditor` window.
    /// Can optionally be serialized/deserialized with `serde`
    pub options: MemoryEditorOptions,
    /// Data for layout between frames, rather hacky.
    frame_data: BetweenFrameUiData,
}

impl<T> MemoryEditor<T> {
    pub fn new(text: impl Into<String>) -> Self {
        MemoryEditor {
            window_name: text.into(),
            read_function: None,
            write_function: None,
            address_space: (0..u16::MAX as usize),
            address_characters: 4,
            read_only: false,
            options: Default::default(),
            frame_data: Default::default()
        }
    }

    /// Create a window and render the memory editor contents within.
    ///
    /// If you want to make your own window/container to be used for the editor contents, you can use `draw_viewer_contents()`.
    pub fn window_ui(&mut self, ctx: &CtxRef, memory: &mut T) {
        assert!(self.read_function.is_some(), "The read function needs to be set before one can run the editor!");
        assert!(self.write_function.is_some() || self.read_only, "The write function needs to be set if not in read only mode!");

        let mut is_open = self.options.is_open;

        Window::new(self.window_name.clone())
            .open(&mut is_open)
            .scroll(false)
            .resizable(true)
            .show(ctx, |ui| {
                self.shrink_window_ui(ui);
                self.draw_viewer_contents(ui, memory);
            });

        self.options.is_open = is_open;
    }

    /// Draws the actual memory viewer/editor.
    ///
    /// Can be included in whatever container you want.
    ///
    /// Use `window_ui()` if you want to have a window with the contents instead.
    pub fn draw_viewer_contents(&mut self, ui: &mut Ui, memory: &mut T) {
        self.draw_options_area(ui);

        let Self {
            options,
            read_function,
            address_space,
            address_characters,
            frame_data,
            ..
        } = self;

        let MemoryEditorOptions {
            show_options,
            data_preview_options,
            show_ascii_sidebar,
            zero_colour,
            column_count,
            address_text_colour,
            memory_editor_text_style,
            ..
        } = options;

        ui.separator();

        // Memory Editor Part.
        let zero_colour = zero_colour.unwrap_or_else(|| ui.style().visuals.text_color());
        let read_function = read_function.as_mut().unwrap();

        let max_lines = address_space.len().div_ceil(column_count);
        let line_height = get_label_line_height(ui, *memory_editor_text_style);

        list_clipper::ClippedScrollArea::auto_sized(max_lines, line_height).show(ui, |ui, line_range| {
            // Memory values and addresses
            egui::Grid::new("mem_edit_grid")
                .striped(true)
                .spacing(Vec2::new(15.0, ui.style().spacing.item_spacing.y))
                .show(ui, |mut ui| {
                    ui.style_mut().body_text_style = *memory_editor_text_style;
                    ui.style_mut().spacing.item_spacing.x = 3.0;

                    for start_row in line_range.clone() {
                        let start_address = address_space.start + (start_row * *column_count);
                        ui.colored_label(*address_text_colour, format!("0x{:01$X}", start_address, address_characters));

                        // Render the memory values
                        for c in 0..column_count.div_ceil(&8) {
                            ui.columns((*column_count - 8 * c).min(8), |columns| {
                                let start_address = start_address + 8 * c;
                                for (i, column) in columns.iter_mut().enumerate() {
                                    let memory_address = start_address + i;

                                    if !address_space.contains(&memory_address) {
                                        break;
                                    }

                                    let mem_val: u8 = read_function(memory, memory_address);

                                    let text_colour = if mem_val == 0 {
                                        zero_colour
                                    } else {
                                        column.style().visuals.text_color()
                                    };

                                    column.add(Label::new(format!("{:02X}", mem_val)).text_color(text_colour));
                                }
                            });
                        }

                        // Optional ASCII side
                        if *show_ascii_sidebar {
                            // Not pretty atm, needs a better method: TODO
                            ui.horizontal(|ui| {
                                ui.add(egui::Separator::new().vertical().spacing(3.0));
                                ui.style_mut().spacing.item_spacing.x = 0.0;
                                ui.columns(*column_count, |columns| {
                                    for (i, column) in columns.iter_mut().enumerate() {
                                        let memory_address = start_address + i;
                                        if !address_space.contains(&memory_address) {
                                            break;
                                        }

                                        let mem_val: u8 = read_function(memory, memory_address);
                                        let character = if mem_val < 32 || mem_val >= 128 { '.' } else { mem_val as char };
                                        column.add(egui::Label::new(character).text_style(TextStyle::Monospace));
                                    }
                                });
                            });
                        }

                        ui.end_row();
                    }


                });

            // After we've drawn the area we want to resize to we want to save this size for the next frame.
            frame_data.previous_frame_editor_width = ui.min_rect().width();
        });
    }

    fn draw_options_area(&mut self, ui: &mut Ui) {
        let Self {
            options,
            read_function,
            address_space,
            address_characters,
            ..
        } = self;

        let MemoryEditorOptions {
            show_options,
            data_preview_options,
            show_ascii_sidebar,
            zero_colour,
            column_count,
            address_text_colour,
            memory_editor_text_style,
            ..
        } = options;

        ui.text_edit_singleline(&mut "Hey".to_string());
        ui.button("Hello World");

        let mut columns = *column_count as u8;
        ui.add(egui::DragValue::u8(&mut columns).range(1.0..=64.0).prefix("Columns: ").speed(0.5));
        *column_count = columns as usize;

        ui.checkbox(show_ascii_sidebar, "Show ASCII");
    }

    /// Shrink the window to the previous frame's memory viewer's width.
    /// This essentially allows us to only have height resize, and have width grow/shrink as appropriate.
    fn shrink_window_ui(&self, ui: &mut Ui) {
        ui.set_max_width(ui.min_rect().width().max(self.frame_data.previous_frame_editor_width));
    }

    /// Set the function used to read from the provided object `T`.
    ///
    /// This function will always be necessary, and the editor will `panic` if it's not available.
    pub fn set_read_function(mut self, read_function: ReadFunction<T>) -> Self {
        self.read_function = Some(read_function);
        self
    }

    /// Set the function used to write to the provided object `T`.
    ///
    /// This function is only necessary if `read_only` is `false`.
    pub fn set_write_function(mut self, write_function: WriteFunction<T>) -> Self {
        self.write_function = Some(write_function);
        self
    }

    /// Set the range of addresses that the UI will display, and subsequently query for using the `read_function`
    pub fn set_address_space(mut self, address_range: Range<usize>) -> Self {
        self.address_space = address_range;
        // This is incredibly janky, but I couldn't find a better way atm. If there is one, please mention it!
        self.address_characters = format!("{:X}", self.address_space.end).chars().count();
        self
    }

    /// If set to `true` the UI will not allow any manual memory edits, and thus the `write_function` will never be called
    /// (and therefore doesn't need to be set).
    pub fn set_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set the memory options, useful if you use the `persistence` feature.
    pub fn with_options(mut self, options: MemoryEditorOptions) -> Self {
        self.options = options;
        self
    }
}
