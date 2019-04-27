extern crate minifb;

pub mod cpu;
pub mod memory;
pub mod opcodes;
pub mod video;

mod clock;

use crate::debugger;
use minifb::*;
use crate::utils;

pub const SCREEN_WIDTH:  usize = 384; // extend 20 pixels left and right for the borders
pub const SCREEN_HEIGHT: usize = 272; // extend 36 pixels top and down for the borders

const CLOCK_FREQ: f64 = 50.0;
pub const PC_REG: u32 = 0xF000;

pub struct Virpc {
    pub main_window: minifb::Window,
    pub program_to_load: String,
    pub memory: memory::MemShared,
    clock:  clock::Clock,
    cpu:  cpu::CPUShared,
    video: video::VideoShared,

    debugger: Option<debugger::Debugger>,
    powered_on: bool,
    boot_complete: bool,
    cycle_count: u32,
}

impl Virpc {
    pub fn new(window_scale: Scale, debugger_on: bool, prg_to_load: &str) -> Virpc {
        let memory = memory::Memory::new_shared();
        let cpu    = cpu::CPU::new_shared(PC_REG);
        let video  = video::Video::new_shared();

        let mut virpc = Virpc {
            main_window: Window::new("VirPC", SCREEN_WIDTH, SCREEN_HEIGHT, WindowOptions { scale: window_scale, ..Default::default() }).unwrap(),
            program_to_load: String::from(prg_to_load),
            memory: memory.clone(), // shared system memory (RAM, ROM, IO registers)
            video: video.clone(),
            clock:  clock::Clock::new(CLOCK_FREQ),
            cpu:  cpu.clone(),
            debugger: if debugger_on { Some(debugger::Debugger::new()) } else { None },
            powered_on: false,
            boot_complete: false,
            cycle_count: 0,
        };

        virpc.main_window.set_position(75, 20);

        // cyclic dependencies are not possible in Rust (yet?), so we have
        // to resort to setting references manually
        virpc.cpu.borrow_mut().set_references(memory.clone());
        virpc.video.borrow_mut().set_references(memory.clone(), cpu.clone());

        drop(video);
        drop(memory);
        drop(cpu);

        virpc
    }

    pub fn reset(&mut self) {
        self.memory.borrow_mut().reset();
        self.cpu.borrow_mut().reset();
    }

    pub fn run(&mut self) {
        if !self.powered_on {
            // $0000 is the power-on reset routine
            self.cpu.borrow_mut().set_pc(0x0000);
            self.powered_on = true;
            if self.powered_on {
                let prg_file = &self.program_to_load.to_owned()[..];

                if prg_file.len() > 0 {
                    self.boot_complete = true; self.load_prg(prg_file);
                }
            }
        }

        // main virpc update - use the clock to time all the operations
        if self.clock.tick() {
            self.cpu.borrow_mut().update();
            self.video.borrow_mut().update(self.cycle_count);

            // update the debugger window if it exists
            match self.debugger {
                Some(ref mut dbg) => {
                    if self.cycle_count % 2 == 0 {
                        dbg.render(&mut self.cpu, &mut self.memory);
                    }
                },
                None => (),
            }
            // redraw the screen and process input on every x cycle
            if self.cycle_count % 20 == 0 {
                let _ = self.main_window.update_with_buffer(&self.video.borrow_mut().window_buffer);
            }

            if self.main_window.is_key_pressed(Key::F12, KeyRepeat::No) {
                self.reset();
            }

            self.cycle_count += 1;
        }
    }

    // *** private functions *** //
    // load a *.prg file
    fn load_prg(&mut self, filename: &str) {
        let prg_data = utils::open_file(filename, 0);
        let start_address: u32 = ((prg_data[1] as u32) << 8) | (prg_data[0] as u32);
        //println!("Loading {} to start location at ${:04x} ({})", filename, start_address, start_address);

        for i in 2..(prg_data.len()) {
            self.memory.borrow_mut().write_byte(start_address + (i as u32) - 2, prg_data[i]);
        }
    }
}