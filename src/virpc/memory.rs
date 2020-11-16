// memory banks
use std::cell::RefCell;
use std::rc::Rc;
//use crate::utils;

pub type MemShared = Rc<RefCell<Memory>>;

pub enum MemType {
    Ram,
    Void,
    //Rom,
    //Io,
}

const MEM_SIZE: usize = 0x080000;

// specific memory bank - RAM, ROM, IO
pub struct MemBank {
    bank_type: MemType, // what am I?
    read_only: bool,    // RAM or ROM?
    offset: u32,        // offset from start of address space
    data: Vec<u8>,
}

impl MemBank {
    pub fn new(mem_type: MemType) -> MemBank {
        let mut mem_bank = MemBank {
            bank_type: mem_type,
            read_only: true,
            offset: 0x0000,
            data: Vec::<u8>::new(),
        };

        match mem_bank.bank_type {
            MemType::Ram => {
                mem_bank.data = Vec::<u8>::with_capacity(MEM_SIZE);
                for _ in 0..MEM_SIZE {
                    mem_bank.data.push(0);
                }
                mem_bank.read_only = false;
            },
            MemType::Void => {                
            }
        }
        mem_bank
    }

    pub fn write(&mut self, addr: u32, val: u8) {
        match self.bank_type {
            MemType::Ram => self.data[(addr - self.offset) as usize] = val,
            MemType::Void => {},
        }
    }

    pub fn read(&mut self, addr: u32) -> u8 {
        match self.bank_type {
            MemType::Ram => self.data[(addr - self.offset) as usize],
            MemType::Void => { 0x0 },
        }
    }    
}


// collective memory storage with all the banks and bank switching support
pub struct Memory {
    ram:     MemBank,
    void:    MemBank,
}

impl Memory {
    pub fn new_shared() -> MemShared {
        Rc::new(RefCell::new(Memory {
            ram:     MemBank::new(MemType::Ram),     // MEM_SIZE
            void:     MemBank::new(MemType::Void),     // Void
        }))
    }
    
    // returns memory bank for current latch setting and address
    pub fn get_bank(&mut self, addr: u32) -> &mut MemBank {
        const RAMSIZE: u32 = (MEM_SIZE-1) as u32;
        match addr {
            0x0000..=RAMSIZE => &mut self.ram,
            _                => &mut self.void,
        }
    }

    // returns specific modifiable memory bank
    pub fn get_ram_bank(&mut self, bank_type: MemType) -> &mut MemBank {
        match bank_type {
            MemType::Ram => &mut self.ram,
            _            => &mut self.void,
        }
    }   
    
    pub fn reset(&mut self) {
        // enable kernal, chargen and basic ROMs
    }

    // Write a byte to memory - returns whether RAM was written (true) or RAM under ROM (false)
    pub fn write_byte(&mut self, addr: u32, value: u8) -> bool {
        self.get_bank(addr).write(addr, value);
        return true;
    }
    
    // Read a byte from memory
    pub fn read_byte(&mut self, addr: u32) -> u8 {
        self.get_bank(addr).read(addr)
    }

    // Read a word from memory (stored in little endian)
    pub fn read_int_le(&mut self, addr: u32) -> u32 {
        let bank = self.get_bank(addr);
        let value_be: u32 = ((bank.read(addr) as u32) << 24 & 0xFF000000) |
                            ((bank.read(addr + 0x0001) as u32) << 16 & 0x00FF0000) |
                            ((bank.read(addr + 0x0002) as u32) << 8 & 0x0000FF00) |
                            ((bank.read(addr + 0x0003) as u32) & 0x000000FF);

        let value_le: u32 = ((value_be << 24) & 0xFF000000) | 
                            ((value_be << 8) & 0x00FF0000) | 
                            ((value_be >> 8) & 0x0000FF00) | 
                            ((value_be >> 24) & 0x000000FF);
        value_le
    }

    // Read a word from memory (stored in little endian)
    pub fn write_int_le(&mut self, addr: u32, value: u32) -> bool {
        let bank = self.get_bank(addr);

        let value_be: u32 = ((value << 24) & 0xFF000000) | 
                            ((value << 8) & 0x00FF0000) | 
                            ((value >> 8) & 0x0000FF00) | 
                            ((value >> 24) & 0x000000FF);

        bank.write(addr,(value_be >> 24 & 0x000000FF) as u8);
        bank.write(addr + 0x0001,(value_be >> 16 & 0x000000FF) as u8);
        bank.write(addr + 0x0002,(value_be >> 8 & 0x000000FF) as u8);
        bank.write(addr + 0x0003,(value_be & 0x000000FF) as u8);
        true
    }

    // *** private functions *** //
}

