use std::ops::RangeInclusive;
use crate::option_data::MemoryEditorOptions;
mod option_data;

/// Reads a value present at the provided address in the object `T`.
///
/// # Arguments
///
/// - `&mut T`: the object on which the read should be performed.
/// - `usize`: The address of the read.
type ReadFunction<T> = Box<dyn Fn(&mut T, usize) -> u8>;
/// Writes the changes the user made to the `T` object.
///
/// # Arguments
///
/// - `&mut T`: the object whose state is to be updated.
/// - `usize`: The address of the intended write.
/// - `u8`: The value set by the user for the provided address.
type WriteFunction<T> = Box<dyn Fn(&mut T, usize, u8)>;


pub struct MemoryEditor<T> {
    /// The name of the `egui` window, can be left blank.
    window_name: String,
    /// The function used for getting the values out of the provided type `T` and displaying them.
    read_function: Option<ReadFunction<T>>,
    /// The function used when attempts are made to change values within the GUI.
    write_function: Option<WriteFunction<T>>,
    /// The range of possible values to be displayed, the GUI will start at the lower bound and go up to the upper bound.
    address_space: RangeInclusive<usize>,
    /// When `true` will disallow any edits, ensuring the `write_function` will never be called.
    /// The latter therefore doesn't need to be set.
    read_only: bool,
    /// A collection of options relevant for the `MemoryEditor` window.
    /// Can optionally be serialized/deserialized with `serde`
    pub options: MemoryEditorOptions
}

impl<T> MemoryEditor<T> {
    pub fn new(text: impl Into<String>) -> Self {
        MemoryEditor{
            window_name: text.into(),
            read_function: None,
            write_function: None,
            address_space: (0..=usize::max_value()),
            read_only: false,
            options: Default::default()
        }
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

    pub fn set_address_space(mut self, address_range: RangeInclusive<usize>) -> Self {
        self.address_space = address_range;
        self
    }

    pub fn set_read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

}

