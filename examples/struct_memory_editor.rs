use eframe::epi;
use eframe::epi::Frame;
use eframe::egui::CtxRef;
use egui_memory_editor::MemoryEditor;

pub fn main() {
    let app = App::default();
    eframe::run_native(Box::new(app));
}


pub struct App {
    mem_editor: MemoryEditor<Memory>,
    memory: Memory,
}

impl Default for App {
    fn default() -> Self {
        let mem_editor = MemoryEditor::<Memory>::new(|mem, address| mem.read_value(address))
            .with_address_range("All", 0..0xFFFF)
            .with_address_range("IO", 0xFF00..0xFF80)
            .with_write_function(|mem, address, value| mem.write_value(address, value))
            .with_window_title("Hello Editor!");
        App {
            mem_editor,
            memory: Default::default()
        }
    }
}

impl epi::App for App {
    fn update(&mut self, ctx: &CtxRef, frame: &mut Frame<'_>) {
        if self.mem_editor.options.is_open {
            self.mem_editor.window_ui(ctx, &mut self.memory);
        }
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