use egui::{Color32, TextStyle};
use std::ops::Range;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum Endianness {
    Big,
    Little,
}

impl Endianness {
    pub fn iter() -> impl Iterator<Item = Endianness> {
        vec![Endianness::Big, Endianness::Little].into_iter()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum DataFormatType {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
}

impl DataFormatType {
    pub fn iter() -> impl Iterator<Item = DataFormatType> {
        use DataFormatType::*;
        vec![U8, U16, U32, U64, I8, I16, I32, I64, F32, F64].into_iter()
    }

    pub const fn bytes_to_read(&self) -> usize {
        use DataFormatType::*;
        match *self {
            U8 | I8 => 1,
            U16 | I16 => 2,
            U32 | I32 | F32 => 4,
            U64 | I64 | F64 => 8,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct DataPreviewOptions {
    pub selected_endianness: Endianness,
    pub selected_data_format: DataFormatType,
}

impl Default for DataPreviewOptions {
    fn default() -> Self {
        DataPreviewOptions {
            selected_endianness: Endianness::Little,
            selected_data_format: DataFormatType::U32,
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct MemoryEditorOptions {
    /// Used to check if the window is open, if you don't use the [`crate::MemoryEditor::window_ui`] call then this is irrelevant.
    pub is_open: bool,
    /// Whether to show the ASCII representation of all the `u8` values in the main UI.
    pub show_ascii_sidebar: bool,
    /// Whether `0x00` values in the main UI should use the [`MemoryEditorOptions::zero_colour`].
    pub show_zero_colour: bool,
    /// Whether the options header is collapsed by default or not.
    /// Default is `false`.
    pub is_options_collapsed: bool,
    /// The options which determine how to interpret selected data, concerning endianness and number type.
    pub data_preview_options: DataPreviewOptions,
    /// The amount of columns for the main UI, this amount directly impacts the possible size of your address space.
    ///
    /// At the moment, you'll at most be able to display the range: `0..2^(24 + log_2(column_count))`.
    pub column_count: usize,
    /// Whether column size can be modified
    /// Default is `true`.
    pub is_resizable_column: bool,
    /// A custom colour for `0x00`. By default will be grey.
    pub zero_colour: Color32,
    /// The colour for address indicators on the very left of the UI.
    pub address_text_colour: Color32,
    /// The highlight colour for both the main UI and the ASCII sidebar.
    /// This will be enabled when you right-click an address, or when using the `goto address` function in the UI.
    pub highlight_colour: Color32,
    /// The [`egui::TextStyle`] for the main UI, indicating the values.
    /// Default is [`egui::TextStyle::Monospace`]
    pub memory_editor_text_style: TextStyle,
    /// The [`egui::TextStyle`] for the addresses in the main UI on the left.
    /// Default is [`egui::TextStyle::Monospace`]
    pub memory_editor_address_text_style: TextStyle,
    /// The [`egui::TextStyle`] for the ASCII values in the right side-bar (if they're enabled).
    /// Default is [`egui::TextStyle::Monospace`]
    pub memory_editor_ascii_text_style: TextStyle,
    /// The selected address range, always applicable, not really relevant for consumers of the editor.
    pub(crate) selected_address_range: String,
}

impl Default for MemoryEditorOptions {
    fn default() -> Self {
        MemoryEditorOptions {
            is_open: true,
            data_preview_options: Default::default(),
            show_ascii_sidebar: true,
            show_zero_colour: true,
            is_options_collapsed: false,
            zero_colour: Color32::from_gray(80),
            is_resizable_column: true,
            column_count: 16,
            address_text_colour: Color32::from_rgb(125, 0, 125),
            highlight_colour: Color32::from_rgb(0, 140, 140),
            memory_editor_text_style: TextStyle::Monospace,
            memory_editor_address_text_style: TextStyle::Monospace,
            memory_editor_ascii_text_style: TextStyle::Monospace,
            selected_address_range: "".to_string(),
        }
    }
}

/// Some extra, non-serializable state for between frames.
#[derive(Debug, Default, Clone)]
pub(crate) struct BetweenFrameData {
    /// Used to ensure we can resize the window in height, but not in width.
    pub previous_frame_editor_width: f32,
    /// The address a user clicked on in the UI in the previous frame, used for DataPreview
    pub selected_edit_address: Option<usize>,
    pub selected_edit_address_string: String,
    pub selected_edit_address_request_focus: bool,

    pub memory_range_combo_box_enabled: bool,

    pub selected_highlight_address: Option<usize>,
    /// Whether to show additional highlights around items after the current selected item when they'd be part
    /// of the value in the data preview section.
    pub show_additional_highlights: bool,

    pub goto_address_string: String,
    pub goto_address_line: Option<usize>,
}

impl BetweenFrameData {
    pub fn set_highlight_address(&mut self, new_address: usize) {
        // We want to be able to unselect it.
        self.selected_highlight_address = if matches!(self.selected_highlight_address, Some(current) if current == new_address) {
            self.goto_address_string.clear();
            None
        } else {
            self.goto_address_string = format!("{:X}", new_address);
            Some(new_address)
        };
    }

    pub fn set_selected_edit_address(&mut self, new_address: Option<usize>, address_space: &Range<usize>) {
        self.selected_edit_address_string.clear();
        if matches!(new_address, Some(address) if address_space.contains(&address)) {
            self.set_highlight_address(new_address.unwrap());
            self.selected_edit_address_request_focus = true;
            self.selected_edit_address = new_address;
        } else {
            self.selected_edit_address = None;
        }
    }

    #[inline]
    pub fn should_highlight(&self, address: usize) -> bool {
        self.selected_highlight_address.map_or(false, |addr| addr == address)
            || self.selected_edit_address.map_or(false, |addr| addr == address)
    }

    pub fn should_subtle_highlight(&self, address: usize, data_format: DataFormatType) -> bool {
        self.show_additional_highlights && self.selected_highlight_address.map_or(false, |addr| {
            (addr..addr+data_format.bytes_to_read()).contains(&address)
        })
    }
}
