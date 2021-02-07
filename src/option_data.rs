use egui::{Color32, TextStyle};

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct MemoryEditorOptions {
    /// Used to check if the window is open, if you don't use the `window_ui()` call then this is irrelevant.
    pub is_open: bool,
    pub show_options: bool,
    pub data_preview_options: DataPreviewOptions,
    pub show_ascii_sidebar: bool,
    /// A custom colour for `0`. By default will be gray, set to `None` to have it be the same text colour as the rest.
    pub zero_colour: Option<Color32>,
    pub column_count: usize,
    pub address_text_colour: Color32,
    pub memory_editor_text_style: TextStyle,
}

impl Default for MemoryEditorOptions {
    fn default() -> Self {
        MemoryEditorOptions {
            is_open: true,
            show_options: true,
            data_preview_options: Default::default(),
            show_ascii_sidebar: true,
            zero_colour: Some(Color32::from_gray(80)),
            column_count: 16,
            address_text_colour: Color32::from_rgb(125, 0, 125),
            memory_editor_text_style: TextStyle::Monospace
        }
    }
}

/// A rather hacky struct to maintain some state between frames for layout purposes
#[derive(Debug, Default, Copy, Clone)]
pub(crate) struct BetweenFrameUiData {
    /// Used to ensure we can resize the window in height, but not in width.
    pub previous_frame_editor_width: f32,
}
