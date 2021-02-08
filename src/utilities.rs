use crate::option_data::{DataFormatType, DataPreviewOptions, Endianness};
use egui::Ui;
use std::convert::TryInto;

/// Turn a provided slice into a decimal [`String`] representing it's value, interpretation is based on the provided
/// [`crate::option_data::DataPreviewOptions`].
///
/// The provided `bytes` slice is expected to have the appropriate amount of bytes, or else the function will panic.
pub fn slice_to_decimal_string(data_preview: DataPreviewOptions, bytes: &[u8]) -> String {
    match data_preview.selected_endianness {
        Endianness::Big => match data_preview.selected_data_format {
            DataFormatType::U8 => u8::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U16 => u16::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U32 => u32::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U64 => u64::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I8 => i8::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I16 => i16::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I32 => i32::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I64 => i64::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::F32 => f32::from_be_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::F64 => f64::from_be_bytes(bytes.try_into().unwrap()).to_string(),
        },
        Endianness::Little => match data_preview.selected_data_format {
            DataFormatType::U8 => u8::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U16 => u16::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U32 => u32::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::U64 => u64::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I8 => i8::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I16 => i16::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I32 => i32::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::I64 => i64::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::F32 => f32::from_le_bytes(bytes.try_into().unwrap()).to_string(),
            DataFormatType::F64 => f64::from_le_bytes(bytes.try_into().unwrap()).to_string(),
        },
    }
}

/// Returns the `(current_scroll, max_scroll)` of the current UI (assuming it is within a [`egui::ScrollArea`]).
/// Taken from the `egui` scrolling demo.
///
/// The `max_scroll` will only be valid if all contents within a [`egui::ScrollArea`] have already been requested.
pub fn egui_get_current_scroll(ui: &Ui) -> (f32, f32) {
    let margin = ui.style().visuals.clip_rect_margin;
    (
        ui.clip_rect().top() - ui.min_rect().top() + margin,
        ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin,
    )
}
