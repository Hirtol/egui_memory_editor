use crate::frame_history::FrameHistory;
use eframe::Frame;
use eframe::NativeOptions;
use egui::{Context, Visuals};
use egui_memory_editor::MemoryEditor;

mod frame_history;

pub fn main() {
    eframe::run_native(
        "MemEditApp",
        NativeOptions::default(),
        Box::new(|cc| Box::new(App::default())),
    );
}

pub struct App {
    mem_editor: MemoryEditor,
    memory: Memory,
    // Not relevant code to this crate, here to show performance at this point in time.
    fh: FrameHistory,
    is_open: bool,
}

impl Default for App {
    fn default() -> Self {
        // Create a memory editor with a variety of ranges, need at least one, but can be as many as you want.
        let mut mem_editor = MemoryEditor::new()
            .with_address_range("All", 0..0xFFFF)
            .with_address_range("IO", 0xFF00..0xFF80)
            .with_window_title("Hello Editor!");
        // At the moment the UI can handle addresses in the range from 0..2^(24 + log_2(column_count)).
        // This is something that'll hopefully be addressed soon to allow for ranges up to 2^64.

        // You can set the column count in the UI, but also here. There are a variety of options available in mem_editor.options
        mem_editor.options.column_count = 16;
        App {
            mem_editor,
            memory: Default::default(),
            fh: Default::default(),
            is_open: true,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {
        ctx.set_visuals(Visuals::dark());
        create_frame_history(ctx, frame, &mut self.fh);

        // This will automatically check for `mem_editor.options.is_open`, so no need to do that here.
        // The write function is optional, if you don't set it the UI will be in read-only mode.
        self.mem_editor.window_ui(
            ctx,
            &mut self.is_open,
            &mut self.memory,
            |mem, address| mem.read_value(address).into(),
            |mem, address, val| mem.write_value(address, val),
        );
        // If your memory changes between frames you'll need to re-render at whatever framerate you want.
        ctx.request_repaint();
    }
}

pub struct Memory {
    memory: Vec<u8>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            memory: vec![0xFF; u16::MAX as usize],
        }
    }
}
impl Memory {
    pub fn read_value(&mut self, address: usize) -> u8 {
        self.memory[address]
    }

    pub fn write_value(&mut self, address: usize, val: u8) {
        self.memory[address] = val
    }
}

fn create_frame_history(ctx: &Context, frame: &Frame, frame_history: &mut FrameHistory) {
    frame_history.on_new_frame(ctx.input().time, frame.info().cpu_usage);
    egui::SidePanel::left("SidePanel").show(ctx, |ui| {
        frame_history.ui(ui);
    });
}
