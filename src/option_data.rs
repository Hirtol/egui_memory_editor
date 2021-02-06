use egui::Color32;

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub enum Endianness {
    Big,
    Little,
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
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
    F64
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct DataPreviewOptions {
    pub show_data_preview: bool,
    pub selected_endianness: Endianness,
    pub selected_data_format: DataFormatType,
}

impl Default for DataPreviewOptions {
    fn default() -> Self {
        DataPreviewOptions{
            show_data_preview: false,
            selected_endianness: Endianness::Little,
            selected_data_format: DataFormatType::U32
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "serde_support", derive(serde::Serialize, serde::Deserialize))]
pub struct MemoryEditorOptions {
    pub is_open: bool,
    pub show_options: bool,
    pub data_preview_options: DataPreviewOptions,
    pub show_ascii_sidebar: bool,
    pub grey_out_zeros: bool,
    pub column_count: usize,
    pub address_text_colour: Color32
}

impl Default for MemoryEditorOptions {
    fn default() -> Self {
        MemoryEditorOptions {
            is_open: true,
            show_options: true,
            data_preview_options: Default::default(),
            show_ascii_sidebar: true,
            grey_out_zeros: true,
            column_count: 16,
            address_text_colour: Color32::from_rgb(125, 0, 125)
        }
    }
}