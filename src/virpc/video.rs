use crate::virpc;
use crate::virpc::memory;
use crate::virpc::cpu;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;


pub type VideoShared = Rc<RefCell<Video>>;

pub struct Video {
    pub window_buffer: Vec<u32>,
    mem_ref: Option<memory::MemShared>,
    cpu_ref: Option<cpu::CPUShared>,
    //screen_chunk_offset: usize, // current offset from screen start
    //line_start_offset: usize,   // offset to the next line start on screen
}

impl Video {
    pub fn new_shared() -> VideoShared {
        Rc::new(RefCell::new(Video {
            window_buffer: vec![0; virpc::SCREEN_WIDTH * virpc::SCREEN_HEIGHT],
            mem_ref: None,
            cpu_ref: None,
            //screen_chunk_offset: 0,
            //line_start_offset:   0,
        }))
    }
    
    pub fn update(&mut self, c64_cycle_cnt: u32) -> bool {
        for i in 0..(300*200) {
            let dst_color = as_mut!(self.mem_ref).read_byte(0x10000 + i as u32);
            let color_rgba = utils::fetch_c64_color_rgba(dst_color + (c64_cycle_cnt/100) as u8);
            utils::memset8(&mut self.window_buffer, i + (384*36) + 42 + (84 * (i/300)) , color_rgba);
        }
        false
    }

    pub fn set_references(&mut self, memref: memory::MemShared, cpuref: cpu::CPUShared) {
        self.mem_ref = Some(memref);
        self.cpu_ref = Some(cpuref);
    }
    
    /*
    fn draw_background(&mut self) {
        let dst_color: u8;
        
        dst_color = (self.screen_chunk_offset % 255) as u8;
        
        let color_rgba = utils::fetch_c64_color_rgba(dst_color);
        utils::memset8(&mut self.window_buffer, self.screen_chunk_offset, color_rgba);
    }*/
    

    // *** helper functions for draw_graphics ***
    /*fn draw_std(&mut self, color: &[u8]) {
        let screen_pos = self.screen_chunk_offset + self.x_scroll as usize;
        
        let mut data = self.gfx_data;
        self.window_buffer[screen_pos + 7] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 6] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 5] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 4] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 3] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 2] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos + 1] = utils::fetch_c64_color_rgba(color[(data & 1) as usize]); data >>= 1;
        self.window_buffer[screen_pos    ] = utils::fetch_c64_color_rgba(color[data as usize]);
    }


    fn draw_multi(&mut self, color: &[u8]) {
        let screen_pos = self.screen_chunk_offset + self.x_scroll as usize;

        let mut data = self.gfx_data;
        self.window_buffer[screen_pos + 7] = utils::fetch_c64_color_rgba(color[(data & 3) as usize]); data >>= 2;
        self.window_buffer[screen_pos + 6] = self.window_buffer[screen_pos + 7];
        self.window_buffer[screen_pos + 5] = utils::fetch_c64_color_rgba(color[(data & 3) as usize]); data >>= 2;
        self.window_buffer[screen_pos + 4] = self.window_buffer[screen_pos + 5];
        self.window_buffer[screen_pos + 3] = utils::fetch_c64_color_rgba(color[(data & 3) as usize]); data >>= 2;
        self.window_buffer[screen_pos + 2] = self.window_buffer[screen_pos + 3];
        self.window_buffer[screen_pos + 1] = utils::fetch_c64_color_rgba(color[(data as usize)]);
        self.window_buffer[screen_pos    ] = self.window_buffer[screen_pos + 1];
    }*/
}
