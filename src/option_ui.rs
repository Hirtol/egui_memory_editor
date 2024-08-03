use std::ops::Range;

use egui::Ui;

use crate::option_data::{DataFormatType, DataPreviewOptions, Endianness};
use crate::{Address, MemoryEditor};

impl MemoryEditor {
    /// Draw the `Options` collapsing header with the main options and data preview hidden underneath.
    pub(crate) fn draw_options_area<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        mem: &mut T,
        read: &mut impl FnMut(&mut T, Address) -> Option<u8>,
    ) {
        let current_address_range = self
            .address_ranges
            .get(&self.options.selected_address_range)
            .unwrap()
            .clone();

        egui::CollapsingHeader::new("🛠 Options")
            .default_open(!self.options.is_options_collapsed)
            .show(ui, |ui| {
                self.draw_main_options(ui, &current_address_range);

                self.draw_data_preview(ui, &current_address_range, mem, read);
            });
    }

    /// Draw the main options, including the column selection and goto address.
    fn draw_main_options(&mut self, ui: &mut Ui, current_address_range: &Range<Address>) {
        egui::Grid::new("options_grid").show(ui, |ui| {
            // Memory region selection
            if self.frame_data.memory_range_combo_box_enabled {
                let selected_address_range = &mut self.options.selected_address_range;
                let address_ranges = &self.address_ranges;

                ui.horizontal(|ui| {
                    ui.label("Region:");

                    egui::ComboBox::from_id_source("RegionCombo")
                        .selected_text(selected_address_range.clone())
                        .show_ui(ui, |ui| {
                            address_ranges.iter().for_each(|(range_name, _)| {
                                ui.selectable_value(selected_address_range, range_name.clone(), range_name);
                            });
                        });
                });
            };

            // Column dragger
            let mut columns_u8 = self.options.column_count as u8;

            if self.options.is_resizable_column {
                ui.add(
                    egui::DragValue::new(&mut columns_u8)
                        .range(1.0..=64.0)
                        .prefix("Columns: ")
                        .speed(0.5),
                );
            } else {
                ui.add(egui::Label::new(format!("Columns: {}", columns_u8)));
            }

            self.options.column_count = columns_u8 as usize;

            // Goto address
            let response = ui
                .add_sized(
                    ui.available_size(),
                    egui::TextEdit::singleline(&mut self.frame_data.goto_address_string).hint_text("0000"),
                )
                .on_hover_text(
                    "Goto an address, format: \n\
                    * An address like `0xAA` can be written as `AA`\n\
                    * Offset from the base address, if the base is `0xFF00` then one can enter `5` to go to `0xFF05`\n\
                    Press enter to move to the address",
                );
            ui.label(format!("Goto: {:#X?}", current_address_range));

            self.frame_data.goto_address_string.retain(|c| c.is_ascii_hexdigit());

            // For some reason egui is triggering response.clicked() when we press enter at the moment
            // (didn't used to do this). The additional check for not having enter pressed will need to stay until that is fixed.
            if response.clicked() && !ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                self.frame_data.goto_address_string.clear();
            }

            // If we pressed enter, move to the address
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let goto_address_string = &mut self.frame_data.goto_address_string;

                if goto_address_string.starts_with("0x") || goto_address_string.starts_with("0X") {
                    *goto_address_string = goto_address_string[2..].to_string();
                }

                let address = Address::from_str_radix(goto_address_string, 16).ok().and_then(|addr| {
                    if current_address_range.contains(&addr) {
                        Some(addr)
                    } else {
                        // For brevity the user should be able to elide the base address, e.g when using the range
                        // 0xFF00..0xFFFF the user can write 0x5 to go to 0xFF05
                        let offset_addr = addr.saturating_add(current_address_range.start);

                        if current_address_range.contains(&offset_addr) {
                            // We're doing an offset jump, we should update the string to reflect the absolute address
                            *goto_address_string = format!("{:X}", offset_addr);
                            Some(offset_addr)
                        } else {
                            None
                        }
                    }
                });

                self.frame_data.goto_address_line = address
                    .and_then(|addr| addr.checked_sub(current_address_range.start))
                    .map(|addr| addr / self.options.column_count);
                self.frame_data.selected_highlight_address = address;

                response.surrender_focus();
            }

            ui.end_row();

            // Checkboxes
            let show_ascii_sidebar = &mut self.options.show_ascii;
            let show_zero_colour = &mut self.options.show_zero_colour;

            ui.checkbox(show_ascii_sidebar, "Show ASCII").on_hover_text(format!(
                "{} the ASCII representation view",
                if *show_ascii_sidebar { "Disable" } else { "Enable" }
            ));

            ui.checkbox(show_zero_colour, "Custom zero colour")
                .on_hover_text("If enabled memory values of '0x00' will be coloured differently");
        });
    }

    /// Draws the data preview underneath a collapsing header.
    fn draw_data_preview<T: ?Sized>(
        &mut self,
        ui: &mut Ui,
        current_address_range: &Range<Address>,
        mem: &mut T,
        read: &mut impl FnMut(&mut T, Address) -> Option<u8>,
    ) {
        let response = egui::CollapsingHeader::new("⛃ Data Preview")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("data_preview_grid").show(ui, |ui| {
                    let data_preview_options = &mut self.options.data_preview;
                    // Format selection
                    egui::ComboBox::from_label("Endianness")
                        .selected_text(format!("{:?}", data_preview_options.selected_endianness))
                        .show_ui(ui, |ui| {
                            for endian in Endianness::iter() {
                                ui.selectable_value(
                                    &mut data_preview_options.selected_endianness,
                                    endian,
                                    format!("{:?}", endian),
                                );
                            }
                        })
                        .response
                        .on_hover_text("Select the endianness of the data");

                    egui::ComboBox::from_label("Format")
                        .selected_text(format!("{:?}", data_preview_options.selected_data_format))
                        .show_ui(ui, |ui| {
                            for format in DataFormatType::iter() {
                                ui.selectable_value(
                                    &mut data_preview_options.selected_data_format,
                                    format,
                                    format!("{:?}", format),
                                );
                            }
                        })
                        .response
                        .on_hover_text("Select the number type for data interpretation");

                    ui.end_row();

                    // Read and display the value
                    let hover_text = "Right click a value in the UI to select it, right click again to unselect";

                    if let Some(address) = self.frame_data.selected_highlight_address {
                        let value =
                            Self::read_mem_value(mem, read, address, *data_preview_options, current_address_range);
                        ui.label(format!("Value at {:#X} (decimal): ", address))
                            .on_hover_text(hover_text);
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

    fn read_mem_value<T: ?Sized>(
        mem: &mut T,
        read_fn: &mut impl FnMut(&mut T, Address) -> Option<u8>,
        address: Address,
        data_preview: DataPreviewOptions,
        address_space: &Range<Address>,
    ) -> String {
        let bytes = (0..data_preview.selected_data_format.bytes_to_read())
            .map(|i| {
                let read_address = address + i;
                if address_space.contains(&read_address) {
                    read_fn(mem, read_address).unwrap_or(0)
                } else {
                    0
                }
            })
            .collect::<Vec<u8>>();

        crate::utilities::slice_to_decimal_string(data_preview, &bytes)
    }
}
