use egui::{Label, TextStyle, Ui};

/// Returns the `(current_scroll, max_scroll)` of the current UI (assuming it is within a `ScrollArea`).
/// Taken from the `egui` scrolling demo.
///
/// The `max_scroll` will only be valid if all contents within a `ScrollArea` have already been requested.
pub fn get_current_scroll(ui: &Ui) -> (f32, f32) {
    let margin = ui.style().visuals.clip_rect_margin;
    (
        ui.clip_rect().top() - ui.min_rect().top() + margin,
        ui.min_rect().height() - ui.clip_rect().height() + 2.0 * margin,
    )
}

/// Return the line height for the current provided `Ui` and `TextStyle`
pub fn get_label_line_height(ui: &Ui, style: TextStyle) -> f32 {
    Label::new("##invisible").text_style(style).layout(ui).size.y + ui.style().spacing.item_spacing.y
}
