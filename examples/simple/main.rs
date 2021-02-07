use eframe::epi;
use eframe::epi::Frame;
use eframe::egui::CtxRef;
use egui_memory_editor::MemoryEditor;
use crate::frame_history::FrameHistory;

mod frame_history;

pub fn main() {
    let app = App::default();
    eframe::run_native(Box::new(app));
}

pub struct App {
    mem_editor: MemoryEditor<Memory>,
    memory: Memory,
    // Not relevant code to this crate, here to show performance at this point in time.
    fh: FrameHistory,
}

impl Default for App {
    fn default() -> Self {
        // Create a memory editor with a variety of ranges, need at least one, but can be as many as you want.
        // The write function is optional, if you don't set it the UI will be in read-only mode.
        let mem_editor = MemoryEditor::<Memory>::new(|mem, address| mem.read_value(address))
            .with_address_range("All", 0..0xFFFF)
            .with_address_range("IO", 0xFF00..0xFF80)
            .with_write_function(|mem, address, value| mem.write_value(address, value))
            .with_window_title("Hello Editor!");
        App {
            mem_editor,
            memory: Default::default(),
            fh: Default::default()
        }
    }
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        create_frame_history(ctx, frame, &mut self.fh);

        // This will automatically check for `mem_editor.options.is_open`, so no need to do that here.
        self.mem_editor.window_ui(ctx, &mut self.memory);
        // If your memory changes between frames you'll need to rerender at whatever rate you want.
        ctx.request_repaint();
    }

    fn name(&self) -> &str {
        "Mem-edit Example"
    }
}

#[derive(Clone)]
pub struct Memory {
    memory: Vec<u8>,
    read_addresses: Vec<usize>,
}

impl Default for Memory {
    fn default() -> Self {
        Memory {
            memory: vec![0xFF; u16::MAX as usize],
            read_addresses: vec![],
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

fn create_frame_history(ctx: &CtxRef, frame: &Frame<'_>, frame_history: &mut FrameHistory) {
    frame_history.on_new_frame(ctx.input().time, frame.info().cpu_usage);
    egui::SidePanel::left("SidePanel", 300.0).show(ctx, |ui| {
        frame_history.ui(ui);
    });
}