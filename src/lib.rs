use std::ops::{RangeInclusive, Range};

use egui::{Color32, CtxRef, Label, Layout, TextStyle, Ui, Vec2, Window};
use num::Integer;

use crate::option_data::MemoryEditorOptions;

mod option_data;

/// Reads a value present at the provided address in the object `T`.
///
/// # Arguments
///
/// - `&mut T`: the object on which the read should be performed.
/// - `usize`: The address of the read.
type ReadFunction<T> = Box<dyn FnMut(&mut T, usize) -> u8>;
/// Writes the changes the user made to the `T` object.
///
/// # Arguments
///
/// - `&mut T`: the object whose state is to be updated.
/// - `usize`: The address of the intended write.
/// - `u8`: The value set by the user for the provided address.
type WriteFunction<T> = Box<dyn FnMut(&mut T, usize, u8)>;


pub struct MemoryEditor<T> {
    /// The name of the `egui` window, can be left blank.
    window_name: String,
    /// The function used for getting the values out of the provided type `T` and displaying them.
    read_function: Option<ReadFunction<T>>,
    /// The function used when attempts are made to change values within the GUI.
    write_function: Option<WriteFunction<T>>,
    /// The range of possible values to be displayed, the GUI will start at the lower bound and go up to the upper bound.
    address_space: Range<usize>,
    /// When `true` will disallow any edits, ensuring the `write_function` will never be called.
    /// The latter therefore doesn't need to be set.
    read_only: bool,
    /// A collection of options relevant for the `MemoryEditor` window.
    /// Can optionally be serialized/deserialized with `serde`
    pub options: MemoryEditorOptions,
}

impl<T> MemoryEditor<T> {
    pub fn new(text: impl Into<String>) -> Self {
        MemoryEditor {
            window_name: text.into(),
            read_function: None,
            write_function: None,
            address_space: (0..usize::max_value()),
            read_only: false,
            options: Default::default(),
        }
    }

    pub fn ui(&mut self, ctx: &CtxRef, memory: &mut T) {
        assert!(self.read_function.is_some(), "The read function needs to be set before one can run the editor!");
        assert!(self.write_function.is_some() || self.read_only, "The write function needs to be set if not in read only mode!");

        let Self {
            options,
            read_function,
            address_space,
            ..
        } = self;

        let MemoryEditorOptions {
            is_open,
            show_options,
            data_preview_options,
            show_ascii_sidebar,
            grey_out_zeros,
            column_count
        } = options;

        let mut read_function = read_function.as_mut().unwrap();

        Window::new(self.window_name.clone())
            .open(is_open)
            .scroll(true)
            .show(ctx, |ui| {
                let clip_rect = ui.clip_rect();
                println!("Min: {:?}", clip_rect);
                println!("Spacing: {:?}", ui.style().spacing);

                egui::Grid::new("mem_edit_grid")
                    .striped(true)
                    .spacing(Vec2::new(15., ui.style().spacing.item_spacing.y))
                    .show(ui, |ui| {
                        ui.style_mut().body_text_style = TextStyle::Monospace;
                        ui.style_mut().spacing.item_spacing.x = 3.0;
                        for start_address in address_space.clone().step_by(*column_count) {
                            let rectangle = ui.add(Label::new(format!("0x{:04X}", start_address))
                                .text_color(Color32::from_rgb(120, 0, 120))
                                .heading()).rect;

                            if clip_rect.intersects(rectangle) {
                                for c in 0..column_count.div_ceil(&8) {
                                    ui.columns(8, |columns| {
                                        let start_address = start_address + 8 * c;
                                        for (i, column) in columns.iter_mut().enumerate() {
                                            let memory_address = (start_address + i);
                                            if !address_space.contains(&memory_address) {
                                                break;
                                            }

                                            let mem_val: u8 = read_function(memory, memory_address);
                                            column.add(Label::new(format!("{:02X}", mem_val)).heading());
                                        }
                                    });
                                }
                            }
                            ui.end_row();
                        }
                        // for i in (0..clip_rect.max.y as usize) {
                        //     let start_address = i * *column_count;
                        //     let rectangle = ui.add(Label::new(format!("0x{:04X}", start_address))
                        //         .text_color(Color32::from_rgb(120, 0, 120))
                        //         .heading()).rect;
                        //     if !clip_rect.intersects(rectangle) {
                        //         break;
                        //     }
                        //     println!("{:?}", rectangle);
                        //
                        //     for c in 0..column_count.div_ceil(&8) {
                        //         ui.columns(8, |columns| {
                        //             let start_address = start_address + 8 * c;
                        //             for (i, column) in columns.iter_mut().enumerate() {
                        //                 let memory_address = (start_address + i);
                        //                 if !address_space.contains(&memory_address) {
                        //                     break;
                        //                 }
                        //
                        //                 let mem_val: u8 = read_function(memory, memory_address);
                        //                 column.add(Label::new(format!("{:02X}", mem_val)).heading());
                        //             }
                        //         });
                        //     }
                        //     ui.end_row();
                        // }
                    });
                ui.text_edit_singleline(&mut "Hey".to_string());
                ui.button("Hello World");
            });
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

    pub fn set_address_space(mut self, address_range: Range<usize>) -> Self {
        self.address_space = address_range;
        self
    }

    pub fn set_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }
}

