// The CPU
use crate::virpc::memory;
use crate::virpc::opcodes;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;

pub type CPUShared = Rc<RefCell<CPU>>;

pub const NMI_VECTOR:   u32 = 0x0000FFFA;
pub const RESET_VECTOR: u32 = 0x0000FFFC;
pub const IRQ_VECTOR:   u32 = 0x0000FFFE;

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

// action to perform on specific CIA and VIC events
pub enum Callback {
    None,
    TriggerVICIrq,
    ClearVICIrq,
    TriggerCIAIrq,
    ClearCIAIrq,
    TriggerNMI,
    ClearNMI
}

pub enum CPUState {
    FetchOp,
    FetchOperandAddr,
    ProcessIRQ,
    ProcessNMI,
    ExecuteOp
}


pub struct CPU {
    pub pc: u32, // program counter
    pub sp: u32,  // stack pointer
    pub p:  u8,  // processor status
    pub mem_ref:  Option<memory::MemShared>, // reference to shared system memory

    pub instruction: opcodes::Instruction,
    pub cia_irq: bool,
    pub vic_irq: bool,
    pub irq_cycles_left: u8,
    pub nmi_cycles_left: u8,
    pub first_nmi_cycle: u32,
    pub first_irq_cycle: u32,
    pub state: CPUState,
    pub nmi: bool,
    pub debug_instr: bool,
    pub prev_pc: u32, // previous program counter - used for debugging
    pub op_debugger: utils::OpDebugger,
    dfff_byte: u8
}

impl CPU {
    pub fn new_shared() -> CPUShared {
        Rc::new(RefCell::new(CPU {
            pc: 0,
            sp: 0xFF,
            p:  0,
            mem_ref:  None,
            cia_irq: false,
            vic_irq: false,
            irq_cycles_left: 0,
            nmi_cycles_left: 0,
            first_nmi_cycle: 0,
            first_irq_cycle: 0,
            state: CPUState::FetchOp,
            instruction: opcodes::Instruction::new(),
            nmi: false,
            debug_instr: false,
            prev_pc: 0,
            op_debugger: utils::OpDebugger::new(),
            dfff_byte: 0x55
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


    pub fn update(&mut self, c64_cycle_cnt: u32) {
        // check for irq and nmi
        match self.state {
            CPUState::FetchOp => {
                if self.nmi && self.nmi_cycles_left == 0 && (c64_cycle_cnt - (self.first_nmi_cycle as u32) >= 2) {
                    self.nmi_cycles_left = 7;
                    self.state = CPUState::ProcessNMI;
                }
                else if !self.get_status_flag(StatusFlag::InterruptDisable) {
                    let irq_ready = (self.cia_irq || self.vic_irq) && self.irq_cycles_left == 0;

                    if irq_ready && (c64_cycle_cnt - (self.first_irq_cycle as u32) >= 2) {
                        self.irq_cycles_left = 7;
                        self.state = CPUState::ProcessIRQ;
                    }
                }
            },
            _ => {}
        }
        
        match self.state {
            CPUState::FetchOp => {
                let next_op = self.next_byte(); //retrieve next byte
                match opcodes::get_instruction(next_op) { //retrieve instruction
                    Some((opcode, size, arguments, addr_type)) => {
                        self.instruction.opcode = opcode;
                        self.instruction.size = size;
                        self.instruction.args = arguments;
                        self.instruction.addressing_type = addr_type;
                        if self.debug_instr { utils::debug_instruction(next_op, self); }
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
            CPUState::ProcessIRQ => {
                if self.process_irq(false) {
                    self.cia_irq = false;
                    self.vic_irq = false;
                    self.state = CPUState::FetchOp;
                }
            },
            CPUState::ProcessNMI => {
                if self.process_irq(true) {
                    self.nmi = false;
                    self.state = CPUState::FetchOp;
                }
            },
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


    // stack memory: $0100 - $01FF (256 byes)
    pub fn push_byte(&mut self, value: u8) {
        self.sp -= 0x01;
        let new_sp = (self.sp + 0x01) as u32;
        self.write_byte(0x0100 + new_sp, value);
    }


    pub fn pop_byte(&mut self) -> u8 {
        let addr = 0x0100 + (self.sp + 0x01) as u32;
        let value = self.read_byte(addr);
        self.sp += 0x01;
        value
    }


    pub fn push_word(&mut self, value: u32) {
        self.push_byte(((value >> 24) & 0xFF) as u8);
        self.push_byte(((value >> 16) & 0xFF) as u8);
        self.push_byte(((value >> 8) & 0xFF) as u8);
        self.push_byte((value & 0xFF) as u8);
    }


    pub fn write_byte(&mut self, addr: u32, value: u8) -> bool {
        let mut on_write = Callback::None;
        let mut mem_write_ok = true;

        mem_write_ok = as_mut!(self.mem_ref).write_byte(addr, value);

        // on VIC/CIA register write perform necessary action on the CPU
        match on_write {
            Callback::TriggerVICIrq => self.set_vic_irq(true),
            Callback::ClearVICIrq   => self.set_vic_irq(false),
            Callback::TriggerCIAIrq => self.set_cia_irq(true),
            Callback::ClearCIAIrq   => self.set_cia_irq(false),
            Callback::TriggerNMI    => self.set_nmi(true),
            Callback::ClearNMI      => self.set_nmi(false),
            _ => (),
        }

        mem_write_ok
    }
    

    pub fn read_byte(&mut self, addr: u32) -> u8 {
        let byte: u8;
        let mut on_read = Callback::None;

        byte = as_mut!(self.mem_ref).read_byte(addr);

        match on_read {
            Callback::TriggerCIAIrq => self.set_cia_irq(true),
            Callback::ClearCIAIrq   => self.set_cia_irq(false),
            Callback::TriggerNMI    => self.set_nmi(true),
            Callback::ClearNMI      => self.set_nmi(false),
            _ => (),
        }

        byte
    }


    pub fn read_int_le(&self, addr: u32) -> u32 {
        as_ref!(self.mem_ref).read_int_le(addr)
    }

    pub fn write_int_le(&self, addr: u32,value: u32) -> bool {
        as_ref!(self.mem_ref).write_int_le(addr,value)
    }


    pub fn set_vic_irq(&mut self, val: bool) {
        self.vic_irq = val;
    }


    pub fn set_nmi(&mut self, val: bool) {
        self.nmi = val;
    }


    pub fn set_cia_irq(&mut self, val: bool) {
        self.cia_irq = val;
    }


    // *** private functions *** //

    fn process_irq(&mut self, is_nmi: bool) -> bool {
        let new_pc    = if is_nmi { NMI_VECTOR } else { IRQ_VECTOR };
        let cycle_cnt = if is_nmi { self.nmi_cycles_left } else { self.irq_cycles_left };
        
        match cycle_cnt {
            7 | 6 => {
            },
            5 => {
                let pc_hi = (self.pc >> 8) as u8;
                self.push_byte(pc_hi);
            },
            4 => {
                let pc_lo = self.pc as u8;
                self.push_byte(pc_lo);
            },
            3 => {
                self.set_status_flag(StatusFlag::Break, false);
                let curr_p = self.p;
                self.push_byte(curr_p);
                self.set_status_flag(StatusFlag::InterruptDisable, true);
            },
            2 => {
            },
            1 => {
                self.pc = as_ref!(self.mem_ref).read_int_le(new_pc);
            }
            _ => panic!("Invalid IRQ/NMI cycle")
        }

        if is_nmi {
            self.nmi_cycles_left -= 1;
            self.nmi_cycles_left == 0
        }
        else {
            self.irq_cycles_left -= 1;
            self.irq_cycles_left == 0
        }
    }
}
