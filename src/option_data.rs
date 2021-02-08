use egui::{Color32, TextStyle};

pub(crate) const DEFAULT_RANGE_NAME: &str = "DEFAULT";

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum Endianness {
    Big,
    Little,
}

impl Endianness {
    pub fn iter() -> impl Iterator<Item=Endianness> {
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
    pub fn iter() -> impl Iterator<Item=DataFormatType> {
        use DataFormatType::*;
        vec![U8, U16, U32, U64, I8, I16, I32, I64, F32, F64].into_iter()
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct DataPreviewOptions {
    pub show_data_preview: bool,
    pub selected_endianness: Endianness,
    pub selected_data_format: DataFormatType,
}

impl Default for DataPreviewOptions {
    fn default() -> Self {
        DataPreviewOptions {
            show_data_preview: false,
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
    pub show_ascii_sidebar: bool,
    pub show_zero_colour: bool,
    pub data_preview_options: DataPreviewOptions,
    pub column_count: usize,
    /// A custom colour for `0x00`. By default will be grey.
    pub zero_colour: Color32,
    pub address_text_colour: Color32,
    pub highlight_colour: Color32,
    pub memory_editor_text_style: TextStyle,
    pub memory_editor_address_text_style: TextStyle,
    pub memory_editor_ascii_text_style: TextStyle,
    pub(crate) memory_range_combo_box_enabled: bool,
    pub(crate) selected_address_range: String,
    pub(crate) goto_address_string: String,
    pub(crate) goto_address_line: Option<usize>,
}

impl Default for MemoryEditorOptions {
    fn default() -> Self {
        MemoryEditorOptions {
            is_open: true,
            data_preview_options: Default::default(),
            show_ascii_sidebar: true,
            show_zero_colour: true,
            zero_colour: Color32::from_gray(80),
            column_count: 16,
            address_text_colour: Color32::from_rgb(125, 0, 125),
            highlight_colour: Color32::from_rgb(0, 140, 140),
            memory_editor_text_style: TextStyle::Heading, // Non-monospace default as I personally find it too small, and columns provide close-enough alignment.
            memory_editor_address_text_style: TextStyle::Heading,
            memory_editor_ascii_text_style: TextStyle::Monospace,
            memory_range_combo_box_enabled: false,
            selected_address_range: DEFAULT_RANGE_NAME.to_string(),
            goto_address_string: "".to_string(),
            goto_address_line: None,
        }
    }
}

/// A rather hacky struct to maintain some state between frames for layout purposes
#[derive(Debug, Default, Clone)]
pub(crate) struct BetweenFrameUiData {
    /// Used to ensure we can resize the window in height, but not in width.
    pub previous_frame_editor_width: f32,
    /// The address a user clicked on in the UI in the previous frame, used for DataPreview
    pub selected_address: Option<usize>,
    pub selected_address_string: String,
    pub selected_address_request_focus: bool,
}
