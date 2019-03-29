// memory debug window
extern crate minifb;

mod font;

use crate::virpc;
use minifb::*;
use std::io::Write;
use crate::utils;

const DEBUG_W: usize = 640;
const DEBUG_H: usize = 432;
const RASTER_DEBUG_W: usize = 3*63;
const RASTER_DEBUG_H: usize = 312;

// color constants for VIC raster window
const BORDER_COLOR: u32    = 0x00404040;
const BG_COLOR: u32        = 0x00000000;
const VIC_WRITE_COLOR: u32 = 0x00FF0000;
const RASTER_COLOR: u32    = 0x000000FF;
const BADLINE_COLOR: u32   = 0x0000FF00;


pub struct Debugger {
    debug_window: minifb::Window,
    font: font::SysFont,
    window_buffer: Vec<u32>, // main debugger window data buffer
    mempage_offset: u32,     // RAM preview memory page offset
    draw_mode: u8,
}

impl Debugger {
    pub fn new() -> Debugger {
        let mut dbg = Debugger {
            debug_window: Window::new("Debug window", DEBUG_W, DEBUG_H, WindowOptions { scale: Scale::X2, ..Default::default() }).unwrap(),
            font: font::SysFont::new(),
            window_buffer: vec![0; DEBUG_W * DEBUG_H],
            mempage_offset: 0,
            draw_mode: 0,
        };

        dbg.debug_window.set_position(480, 20);

        for y in 1..26 {
            for x in 0..40 {
                dbg.font.draw_char_rgb(&mut dbg.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, 102, 0x00101010);
            }
        }
        dbg
    }


    pub fn render(&mut self, cpu: &mut virpc::cpu::CPUShared, memory: &mut virpc::memory::MemShared) {
        if self.debug_window.is_open() {
            self.draw_border();

            self.draw_ram(memory);
            self.draw_cpu(cpu);

            let _ = self.debug_window.update_with_buffer(&self.window_buffer);
        }
    }


    // *** private functions *** //

    // dump RAM page to screen
    fn draw_ram(&mut self, memory: &mut virpc::memory::MemShared) {
        if self.debug_window.is_key_pressed(Key::PageUp, KeyRepeat::Yes) {
            self.mempage_offset += 0x400;

            if self.mempage_offset > 0xFC00 {
                self.mempage_offset = 0;
            }
        }
        if self.debug_window.is_key_pressed(Key::PageDown, KeyRepeat::Yes) {
            if self.mempage_offset == 0x0000 {
                self.mempage_offset = 0x10000;
            }
            self.mempage_offset -= 0x400;
        }

        let mut start = 0x0000 + self.mempage_offset as u32;
        let mut title = Vec::new();
        let mut hex_offset_x = 0;
        let _ = write!(&mut title, "Memory page ${:04x}-${:04x}", start, start + 0x3FF);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 0, 0, &String::from_utf8(title).unwrap().to_owned()[..], 0x0A);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 34, 0, "*RAM*", 0x0E);

        for y in 0..26 {
            for x in 0..40 {
                let byte = memory.borrow_mut().get_ram_bank(virpc::memory::MemType::Ram).read(start);
                self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8 + 8*y as usize, byte, 0x05);

                self.draw_hex(hex_offset_x + x as usize, 28 + y as usize, byte);
                hex_offset_x += 1;
                start += 1;

                if start == (self.mempage_offset as u32 + 0x0400) { return; }
            }
            hex_offset_x = 0;
        }
    }


    // draw colored hex value of memory cell
    fn draw_hex(&mut self, x_pos: usize, y_pos: usize, byte: u8 ) {
        let mut hex_value = Vec::new();
        let _ = write!(&mut hex_value, "{:02X}", byte);
        
        let mut base_color = utils::fetch_c64_color_rgba(byte >> 4);
        if base_color == 0 {
            base_color = 0x00333333;
        }
        
        // all black? make it at least somewhat visible
        if byte == 0 {
            base_color = 0x00101010;
        }
        
        self.font.draw_text_rgb(&mut self.window_buffer, DEBUG_W, x_pos, y_pos, &String::from_utf8(hex_value).unwrap().to_owned()[..], base_color);        
    } 


    // draw CPU flags and registers
    fn draw_cpu(&mut self, cpu: &mut virpc::cpu::CPUShared) {
        let mut pc_txt = Vec::new();
        let mut p_txt = Vec::new();
        let _ = write!(&mut pc_txt, "${:04X}", cpu.borrow_mut().pc);
        let _ = write!(&mut p_txt, "[{:08b}]", cpu.borrow_mut().p);
        
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 44, 22, "PC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 47, 22, &String::from_utf8(pc_txt).unwrap().to_owned()[..], 0x0E);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 51, 23, "NV-BDIZC:", 0x0F);
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, 61, 23, &String::from_utf8(p_txt).unwrap().to_owned()[..], 0x0E);
    }


    // draw window border
    fn draw_border(&mut self) {
        for x in 0..80 {
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 0, 64, 0x0B);
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*x as usize, 8*27, 64, 0x0B);
        }
        
        for y in 1..27 {
            self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 8*y as usize, 66, 0x0B);
        }

        self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 0, 114, 0x0B);
        self.font.draw_char(&mut self.window_buffer, DEBUG_W, 8*40, 8*27, 113, 0x0B);
    }


    fn clear_char(&mut self, x_pos: usize, y_pos: usize) {
        self.font.draw_text(&mut self.window_buffer, DEBUG_W, x_pos, y_pos, " ", 0x00);
    }


    fn mix_colors(&self, new: u32, old: u32, alpha: f32) -> u32 {
        let rn = ((new >> 16) & 0xFF) as f32;
        let gn = ((new >> 8) & 0xFF) as f32;
        let bn = (new & 0xFF) as f32;

        let ro = ((old >> 16) & 0xFF) as f32;
        let go = ((old >> 8) & 0xFF) as f32;
        let bo = (old & 0xFF) as f32;

        let rd = alpha * rn + (1.0 - alpha) * ro;
        let gd = alpha * gn + (1.0 - alpha) * go;
        let bd = alpha * bn + (1.0 - alpha) * bo;

        let mut dst_color = (rd as u32) << 16;
        dst_color |= (gd as u32) << 8;
        dst_color |= bd as u32;

        dst_color
    }
}
