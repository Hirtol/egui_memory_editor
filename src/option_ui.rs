use std::ops::Range;

use egui::Ui;

use crate::{MemoryEditor, ReadFunction};
use crate::option_data::{DataFormatType, DataPreviewOptions, Endianness};

impl<T> MemoryEditor<T> {
    /// Draw the `Options` collapsing header with the main options and data preview hidden underneath.
    pub(crate) fn draw_options_area(&mut self, ui: &mut Ui, memory: &mut T) {
        let current_address_range = self.address_ranges
            .get(&self.options.selected_address_range)
            .unwrap()
            .clone();

        egui::CollapsingHeader::new("🛠 Options")
            .default_open(true)
            .show(ui, |ui| {
                self.draw_main_options(ui, &current_address_range);

                self.draw_data_preview(ui, &current_address_range, memory);
            });
    }

    /// Draw the main options, including the column selection and goto address.
    fn draw_main_options(&mut self, ui: &mut Ui, current_address_range: &Range<usize>) {
        egui::Grid::new("options_grid").show(ui, |ui| {
            // Memory region selection
            if self.frame_data.memory_range_combo_box_enabled {
                let selected_address_range = &mut self.options.selected_address_range;
                let address_ranges = &self.address_ranges;
                egui::combo_box_with_label(ui, "Memory Region", selected_address_range.clone(), |ui| {
                    address_ranges.iter().for_each(|(range_name, _)| {
                        ui.selectable_value(selected_address_range, range_name.clone(), range_name);
                    });
                });
            }

            // Column dragger
            let mut columns_u8 = self.options.column_count as u8;
            ui.add(
                egui::DragValue::u8(&mut columns_u8)
                    .clamp_range(1.0..=64.0)
                    .prefix("Columns: ")
                    .speed(0.5),
            );
            self.options.column_count = columns_u8 as usize;

            // Goto address
            let response = ui.add(egui::TextEdit::singleline(&mut self.frame_data.goto_address_string).hint_text("0000"))
                .on_hover_text("Goto an address, an address like 0xAA is written as AA\nPress enter to move to the address");
            ui.label(format!("Goto: {:#X?}", current_address_range));

            if response.clicked() {
                self.frame_data.goto_address_string.clear();
            }

            self.frame_data.goto_address_string.retain(|c| c.is_ascii_hexdigit());

            if response.lost_kb_focus() && ui.input().key_pressed(egui::Key::Enter) {
                let goto_address_string = &mut self.frame_data.goto_address_string;
                if goto_address_string.starts_with("0x") || goto_address_string.starts_with("0X") {
                    *goto_address_string = goto_address_string[2..].to_string();
                }
                let address = usize::from_str_radix(goto_address_string, 16);

                self.frame_data.goto_address_line = address.clone().ok()
                    .map(|addr| (addr - current_address_range.start) / self.options.column_count);
                self.frame_data.selected_highlight_address = address.ok();
            }

            ui.end_row();

            // Checkboxes
            let show_ascii_sidebar = &mut self.options.show_ascii_sidebar;
            let show_zero_colour = &mut self.options.show_zero_colour;

            ui.checkbox(show_ascii_sidebar, "Show ASCII")
                .on_hover_text(format!("{} the ASCII representation view", if *show_ascii_sidebar { "Disable" } else { "Enable" }));
            ui.checkbox(show_zero_colour, "Custom zero colour")
                .on_hover_text("If enabled memory values of '0x00' will be coloured differently");
        });
    }

    /// Draws the data preview underneath a collapsing header.
    fn draw_data_preview(&mut self, ui: &mut Ui, current_address_range: &Range<usize>, memory: &mut T) {
        let response = egui::CollapsingHeader::new("⛃ Data Preview").default_open(false).show(ui, |ui| {
            egui::Grid::new("data_preview_grid").show(ui, |ui| {
                let data_preview_options = &mut self.options.data_preview_options;
                // Format selection
                egui::combo_box_with_label(ui, "Endianness", format!("{:?}", data_preview_options.selected_endianness), |ui| {
                    for endian in Endianness::iter() {
                        ui.selectable_value(&mut data_preview_options.selected_endianness, endian, format!("{:?}", endian));
                    }
                }).on_hover_text("Select the endianness for the data interpretation");
                egui::combo_box_with_label(ui, "Format", format!("{:?}", data_preview_options.selected_data_format), |ui| {
                    for format in DataFormatType::iter() {
                        ui.selectable_value(&mut data_preview_options.selected_data_format, format, format!("{:?}", format));
                    }
                }).on_hover_text("Select the number type for data interpretation");

                ui.end_row();

                // Read and display the value
                let hover_text = "Right click a value in the UI to select it, right click again to unselect";

                if let Some(address) = self.frame_data.selected_highlight_address {
                    let value = Self::read_mem_value(self.read_function, address, *data_preview_options, &current_address_range, memory);
                    ui.label(format!("Value at {:#X} (decimal): ", address)).on_hover_text(hover_text);
                    ui.label(value);
                } else {
                    ui.label("Value (decimal): ").on_hover_text(hover_text);
                    ui.label("None");
                }
            });
        });
        // Currently relies on the header being open_default(false), otherwise we'd enable the highlight when closing the preview!
        if response.header_response.clicked() {
            self.frame_data.show_additional_highlights = !self.frame_data.show_additional_highlights;
        }
    }

    fn read_mem_value(read_function: ReadFunction<T>, address: usize, data_preview: DataPreviewOptions, address_space: &Range<usize>, memory: &mut T) -> String {
        let bytes = (0..data_preview.selected_data_format.bytes_to_read())
            .map(|i| {
                let read_address = address + i;
                if address_space.contains(&read_address) {
                    read_function(memory, read_address)
                } else {
                    0
                }
            })
            .collect::<Vec<u8>>();

        crate::utilities::slice_to_decimal_string(data_preview, &bytes)
    }
}
