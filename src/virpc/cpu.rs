// The CPU
use crate::virpc::memory;
use crate::virpc::opcodes;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;
use std::fmt;

use opcodes::ArgumentSize;

pub type CPUShared = Rc<RefCell<CPU>>;

pub const RESET_VECTOR: u32 = 0x0000FFFC;

// status flags for P register
pub enum StatusFlag {
    Carry            = 1 << 0,
    Zero             = 1 << 1,
    InterruptDisable = 1 << 2,
    DecimalMode      = 1 << 3,
    Break            = 1 << 4,
    Unused           = 1 << 5,
    Overflow         = 1 << 6,
    Negative         = 1 << 7,
}

pub enum CPUState {
    FetchOp,
    FetchOperandAddr,
    ExecuteOp
}


pub struct CPU {
    pub pc: u32, // program counter
    pub p:  u8,  // processor status
    pub mem_ref:  Option<memory::MemShared>, // reference to shared system memory

    pub instruction: opcodes::Instruction,
    pub instruction_u8 : u8,
    pub state: CPUState,
    pub prev_pc: u32,
}

impl CPU {
    pub fn new_shared() -> CPUShared {
        Rc::new(RefCell::new(CPU {
            pc: 0,
            p:  0,
            mem_ref:  None,
            instruction_u8 : 0,
            state: CPUState::FetchOp,
            instruction: opcodes::Instruction::new(),
            prev_pc: 0,
        }))
    }


    pub fn set_references(&mut self, memref: memory::MemShared) {
        self.mem_ref = Some(memref);
    }    
    

    pub fn set_status_flag(&mut self, flag: StatusFlag, value: bool) {
        if value { self.p |=   flag as u8;  }
        else     { self.p &= !(flag as u8); }
    }


    pub fn get_status_flag(&mut self, flag: StatusFlag) -> bool {
        self.p & flag as u8 != 0x00
    }


    // these flags will be set in tandem quite often
    pub fn set_zn_flags(&mut self, value: u8) {
        self.set_status_flag(StatusFlag::Zero, value == 0x00);
        self.set_status_flag(StatusFlag::Negative, (value as i8) < 0);
    }
    
    pub fn set_zn_flags_int(&mut self, value: u32) {
        self.set_status_flag(StatusFlag::Zero, value == 0x00);
        self.set_status_flag(StatusFlag::Negative, (value as i32) < 0);
    }

    pub fn reset(&mut self) {
        let pc = self.read_int_le(RESET_VECTOR);
        self.pc = pc;

        // I'm only doing this to avoid dead code warning :)
        self.set_status_flag(StatusFlag::Unused, false);
    }


    pub fn update(&mut self) {
        match self.state {
            CPUState::FetchOp => {
                let next_op = self.next_byte(); //retrieve next byte
                match opcodes::get_instruction(next_op) { //retrieve instruction
                    Some((opcode, size, arguments, addr_type)) => {
                        self.instruction.opcode = opcode;
                        self.instruction.size = size;
                        self.instruction.args = arguments;
                        self.instruction.addressing_type = addr_type;
                    }
                    None => panic!("Can't fetch instruction")
                }
                self.state = CPUState::FetchOperandAddr;
            },
            CPUState::FetchOperandAddr => {
                if opcodes::fetch_operand_addr(self) {
                    self.state = CPUState::ExecuteOp;
                }
            }
            CPUState::ExecuteOp => {
                if opcodes::run(self) {
                    self.state = CPUState::FetchOp;
                }
            }
        }
    }


    pub fn next_byte(&mut self) -> u8 {
        let pc = self.pc;
        let op = self.read_byte(pc);
        self.pc += 1;
        op
    }

    pub fn next_int(&mut self) -> u32 {
        let pc = self.pc;
        let op = self.read_int_le(pc);
        self.pc += 4;
        op
    }

    pub fn write_byte(&mut self, addr: u32, value: u8) -> bool {
        as_mut!(self.mem_ref).write_byte(addr, value);
        true
    }
    
    pub fn read_byte(&mut self, addr: u32) -> u8 {
        let byte: u8;
        byte = as_mut!(self.mem_ref).read_byte(addr);
        byte
    }


    pub fn read_int_le(&self, addr: u32) -> u32 {
        as_ref!(self.mem_ref).read_int_le(addr)
    }

    pub fn write_int_le(&self, addr: u32,value: u32) -> bool {
        as_ref!(self.mem_ref).write_int_le(addr,value)
    }

    pub fn disassemble(&mut self, address: u32) -> u32 {
        //take a memory address
        //return the opcode and arguments
        
        self.pc = address; //retrieve next byte
        self.prev_pc = self.pc;
        let next_op = self.next_byte();
        self.instruction_u8 = next_op;
        match opcodes::get_instruction(next_op) { //retrieve instruction
            Some((opcode, size, arguments, addr_type)) => {
                self.instruction.opcode = opcode;
                self.instruction.size = size;
                self.instruction.args = arguments;
                self.instruction.addressing_type = addr_type;
            }
            None => panic!("Can't fetch instruction")
        }
        opcodes::pull_operand_addr(self);
        //self.pc = self.pc + 1;

        // try to follow all code paths
        //if addr is referenced in instruction, then mark it as a value (int/byte)
        //  if byte = printable char, and more then 2 bytes in a row, then display then as chars/string
        //if instruction is a jump, and addr > pc, then set pc to the jump
        //if instruction is a conditional jump, and addr > pc, then add jump to list of jumps to follow
        self.pc
    }

    pub fn assemble(&mut self) {
        //take an instruction object, and return the related opcode (and argument bytes), and commit to memory
        //self.instruction.opcode | self.instruction.addr_type | self.instruction.args
        let op = opcodes::get_opcode(self);
        self.instruction_u8 = op;
        self.write_byte(self.pc,op);
        self.pc += 1;
        //self.instruction.arg[](size)
        opcodes::push_operand_addr(self);
        //put @ addr in memory (cannot insert, only overwrite, or complete re-assemble-> fixed code with only label-jumps, data/lib/stack section)
    }

    pub fn text_to_instruction(&mut self, line : &str) {
        //parse text, and return a cpu.instruction object
        // b/i INSTRUCTION arg/[arg] // TODO : should these be values or addresses?
        let mut token = false;
        //line[pos]
        let mut tok = String::from("");
        for c in line.chars() { 
            match c {
                ';' => { break; },
                ',' => { /* next arg*/ },
                '[' => {
                        if token == false {
                            token = true;
                            tok.clear();
                        }
                    },
                ']' => {
                        if token == true {
                            token = false;
                            println!("token:[{}]",tok);
                        }
                    },
                c if c.is_alphabetic() || c.is_numeric() || c == '_' => {
                        if token == false {
                            token = true;
                            tok.clear();
                        }
                        tok.push(c);
                    },
                _ => {
                        if token == true {
                            token = false;
                            println!("token:{}",tok);
                        }
                    },
            }
        }
    }

    pub fn instruction_to_text(&mut self) -> std::string::String {
        let mut s;
        match self.instruction.addressing_type {
            ArgumentSize::Int => {
                s = format!("${:04X}: i{},", self.prev_pc, self.instruction);
                for i in 0..(self.instruction.size as usize) {
                    if (self.instruction.args << i) & 0x04 > 0 {
                        s = format!("{} [{}]",s,self.instruction.arg[i]);
                    }
                    else {
                        s = format!("{} {}",s,self.instruction.arg[i]);
                    }
                }
                s = format!("{}\t",s);
                s = format!("{} {:02X}",s,self.instruction_u8);
                for i in 0..(self.instruction.size as usize) {
                    s = format!("{} {:02X}{:02X}{:02X}{:02X}",s,
                        (self.instruction.arg[i]>>24)&0xff,
                        (self.instruction.arg[i]>>16)&0xff,
                        (self.instruction.arg[i]>>8)&0xff,
                        (self.instruction.arg[i])&0xff);
                }  
            }   
            ArgumentSize::Byte => {
                s = format!("${:04X}: b{},", self.prev_pc, self.instruction);
                for i in 0..(self.instruction.size as usize) {
                    if (self.instruction.args << i) & 0x04 > 0 {
                        s = format!("{} [{}]",s,self.instruction.arg[i] as u8);
                    }
                    else{
                        s = format!("{} {}",s,self.instruction.arg[i] as u8);
                    }
                }  
                s = format!("{}\t",s);
                s = format!("{} {:02X}",s,opcodes::get_opcode(self) as u8);
                for i in 0..(self.instruction.size as usize) {
                    s = format!("{} {:02X}",s,self.instruction.arg[i] as u8);
                }  
            }
        }
        s = format!("{}\n",s);
        s
    }

    pub fn get_instruction_index(&mut self) -> u32 {
        match self.instruction.addressing_type {
            ArgumentSize::Int => (((self.instruction_u8 >> 4) & 0x0F) | 0x10) as u32,
            ArgumentSize::Byte => ((self.instruction_u8 >> 4) & 0x0F) as u32,
        }
    }
}
