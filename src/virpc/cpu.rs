// The CPU
use crate::virpc::memory;
use crate::virpc::opcodes;
use std::cell::RefCell;
use std::rc::Rc;
use crate::utils;
use std::fmt;
use ncurses::*;

use opcodes::ArgumentSize;
use opcodes::Op;

pub type CPUShared = Rc<RefCell<CPU>>;

pub const RESET_VECTOR: u32 = 0x00000000;
pub const REGISTERS: u32 = 0x00000001;//til 0x000000FF
pub const CODE: u32 = 0x00000100;
pub const BSS: u32 = 0x0000E000;
pub const MEMORY: u32 = 0x00010000;
pub const STACK: u32 = 0x00080000;

pub const PC_REG: u32 = 0x01;
pub const STACK_REG: u32 = 0x02;
pub const A_REG: u32 = 0x03;
pub const B_REG: u32 = 0x04;
pub const C_REG: u32 = 0x05;

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
    pub data : Vec<ITEM>,
    pub depth : u32,
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
            data : Vec::new(),
            depth : 0,
        }))
    }

    pub fn destroy(&mut self) {
        
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

        //parse arguments 
        match self.instruction.opcode {
            Op::LDR => {// LDR if pc-rel,push, add to local
                //let mut index = 0;
                if self.instruction.args & 0x04 == 0 { //if arg0 is not a ref
                    //TODO add arg0 to list if arg0 > MEMORY
                }

                //read val from [arg1], store in arg0 (and inc addr2)
                if self.instruction.args & 0x02 > 0 {//if arg1 is ref
                    //TODO add arg1 to list

                    //increment value that c points to, if args==xx0
                    if self.instruction.args & 0x01 == 0 && self.instruction.arg[2] > CODE {
                        //TODO add arg2 to list
                    }//else, is a ref, of a ref, so we'll not follow
                }
                else {//if 2nd arg is not a reference, special case: pc relative, or arg2 relative ldr
                    //read val from pc+const_arg1, store in arg0
                    if self.instruction.arg[2] == 0 {//ldr(a=[b+pc])
                        //TODO add arg1+pc to LABEL
                    } 
                    //arg2 is valid, so use as 'stack'-pointer
                    else {
                        //read val from [addr](+const), and inc addr => pop a/[a]
                        //increment value that c points to, if args==xx0
                        if self.instruction.args & 0x01 == 0 {//pop(a=[[stack+b]++]) = ldr 1 b010,
                            //arg1=const
                            if self.instruction.arg[2] == STACK_REG  {
                                if self.instruction.arg[0] == PC_REG && self.instruction.args & 0x04 == 0 {
                                    self.depth -= 1;
                                }
                                else {
                                    //TODO add local var (arg1+stack)
                                }
                            }
                            else {
                                if self.instruction.arg[2] > CODE {
                                    //TODO add VAR{arg2 + arg1(const)
                                }
                            }
                        }
                        //read val from [addr]+const
                        else {//ldr(a=[b+sp])
                            //TODO value from VAR[arg1+[arg2]]
                        }
                    }                  
                }
            },
            // STR if pc-rel,pop remove from local
            Op::STR => {
                let mut index = 0;
                //store to [arg1], read from arg0 (and dec addr2)
                if self.instruction.args & 0x04 > 0 { //if arg0 is a ref
                    //TODO add arg0 to list
                    self.instruction.arg_index[0] = index;//index of arg in list
                }

                if self.instruction.args & 0x02 > 0 {
                    //TODO add arg1 to list
                    self.instruction.arg_index[1] = index;//index of arg in list
                    if self.instruction.args & 0x01 == 0  && self.instruction.arg[2] > CODE {
                        //TODO add arg2 to list
                        self.instruction.arg_index[2] = index;//index of arg in list
                    }//else, is a ref, of a ref, so we'll not follow
                }
                else {//if 1st arg is not a reference, special case: pc relative str, or arg2 relative
                    if self.instruction.arg[2] == 0 {//PC relative
                        //add arg1+pc to LABEL
                        let s = format!("LABEL_{:08X}",self.instruction.arg[1]+self.pc); 
                        let d = format!("DEPTH{}",self.depth);
                        index = CPU::add_new_item(&mut self.data, new_item(s, d) ); 
                        self.instruction.arg_index[1] = index;//index of arg in list                       
                    }
                    else {//arg2+const
                        if self.instruction.args & 0x01 == 0 {//push 
                            if self.instruction.arg[2] == STACK_REG {
                                // add arg1 local var
                                 let s = format!("LOCALVAR_{:08X}",self.instruction.arg[1]+self.read_int_le(self.instruction.arg[2])); 
                                 let d = format!("DEPTH{}",self.depth);
                                 index = CPU::add_new_item(&mut self.data, new_item(s, d) );
                                 self.instruction.arg_index[2] = index;//index of arg in list
                            }
                            else {
                                if self.instruction.arg[2] > CODE {
                                    // add arg2 to global var 
                                    let s = format!("VAR_{:08X}",self.instruction.arg[1]+self.read_int_le(self.instruction.arg[2])); 
                                    index = CPU::add_new_item(&mut self.data, new_item(s, "DEPTH0".to_string()) );
                                    self.instruction.arg_index[2] = index;//index of arg in list
                                }
                            }                               
                        }
                        else {//str([arg2+sp]=arg0)
                            //value from VAR[arg1+[arg2]]
                            let s = format!("LOCALVAR_{:08X}",self.instruction.arg[1]+self.instruction.arg[2]); 
                            let d = format!("DEPTH{}",self.depth);
                            index = CPU::add_new_item(&mut self.data, new_item(s, d) );
                            self.instruction.arg_index[2] = index;//index of arg in list
                        }
                    }
                }
            },
            // CALL: add if not exist addr(pc-rel, or static) to label-list
            Op::CLL => {
                self.depth += 1;//if opcode is call, increment boundary(depth), return; decrement depth,
                if self.instruction.args & 0x04 == 0 && self.instruction.args & 0x02 == 0 {
                    let s = format!("LABEL_{:08X}",self.instruction.arg[0] + self.instruction.arg[1]);
                    let d = format!("DEPTH{}",self.depth);
                    let index = CPU::add_new_item(&mut self.data, new_item(s, d) );
                    self.instruction.arg_index[0] = index;//index of arg in list
                }
            },
            // JMP, add if not exist addr(pc-rel, or static) to label-list
            Op::JMP => {
                if self.instruction.args & 0x04 == 0 {
                    if self.instruction.arg_index[1] < 9 {
                        let s = format!("LABEL_{:08X}",self.instruction.arg[0]);
                        let d = format!("DEPTH{}",self.depth);
                        let index = CPU::add_new_item(&mut self.data, new_item(s, d) );
                        self.instruction.arg_index[0] = index;//index of arg in list
                    }
                    else {
                        let s = format!("LABEL_{:08X}",self.instruction.arg[0] + self.pc);
                        let d = format!("DEPTH{}",self.depth);
                        let index = CPU::add_new_item(&mut self.data, new_item(s, d) );
                        self.instruction.arg_index[0] = index;//index of arg in list
                    }
                } 
            },
            _ => {//any other opcode, default behaviour
                for i in 0..(self.instruction.size as usize) {
                    if (self.instruction.args << i) & 0x04 > 0 || (i == 0 && self.instruction.args & 0x04 == 0) {//arg 0 is always a ref, subsequent are const or ref
                        let mut index = 0;
                        let d = format!("DEPTH{}",self.depth);
                        match self.instruction.arg[i] {
                            RESET_VECTOR => { index = CPU::add_new_item(&mut self.data, new_item("$RESET", "DEPTH0") ); },
                            REGISTERS...CODE => { let s = format!("REG_{:08X}",self.instruction.arg[i]); index = CPU::add_new_item(&mut self.data, new_item(s, d) );},
                            CODE...BSS => { let s = format!("LABEL_{:08X}",self.instruction.arg[i]); index = CPU::add_new_item(&mut self.data, new_item(s, d) );},
                            BSS...MEMORY => { let s = format!("CONST_{:08X}",self.instruction.arg[i]); index = CPU::add_new_item(&mut self.data, new_item(s, d) );},
                            MEMORY...STACK => { let s = format!("VAR_{:08X}",self.instruction.arg[0]); index = CPU::add_new_item(&mut self.data, new_item(s, "DEPTH0".to_string()) );},
                            _ => {},
                        }
                        self.instruction.arg_index[i] = index;//index of arg in list
                    }
                }
            },
        }
        // try to follow all code paths
        //if addr is referenced in instruction, then mark it as a value (int/byte)
        //  if byte = printable char, and more then 2 bytes in a row, then display then as chars/string
        //if instruction is a jump, and addr > pc, then set pc to the jump
        //if instruction is a conditional jump, and addr > pc, then add jump to list of jumps to follow
        self.pc
    }

    fn add_new_item(items : &mut Vec<ITEM>, new_item : ITEM) -> u32 {
        for i in 0..items.len() {
            if item_name(items[i]) == item_name(new_item) {
                if item_description(items[i]) == item_description(new_item) {
                    return i as u32;
                }
            }
        }
        items.push(new_item);
        items.len() as u32
    }

    /*pub fn retrieve_local(&mut self) -> Vec<ITEM> {
        litems1.push(new_item("aaa"));
        litems1
    }

    pub fn retrieve_global() {
        
    }

    pub fn retrieve_labels() {
        
    }*/

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
                        if self.instruction.arg[i] > RESET_VECTOR && self.instruction.arg[i] < CODE {
                            s = format!("{} Reg[{}]",s,self.instruction.arg[i]); 
                        }
                        else {
                            s = format!("{} [{}]",s,self.instruction.arg[i]);                            
                        }

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

    pub fn get_variables_list(&mut self) -> Vec<ITEM> {
        let mut litems2: Vec<ITEM> = Vec::new();
        litems2.push(new_item("register  (0x00)", "1"));//agree on register range
        litems2.push(new_item("new const  (code)", "1"));//byte or int
        litems2.push(new_item("new local  (stack 0x0000FFFF)", "2"));//agree on stack origin (since last call, with unmatched ret.)
        litems2.push(new_item("new global (heap 0x00010000)", "3"));//agree on heap origin
        litems2.push(new_item("-existing-", "4"));
        litems2
    }

    pub fn get_labels_list(&mut self) -> Vec<ITEM> {
        let mut litems3: Vec<ITEM> = Vec::new();
        litems3.push(new_item("new label", ""));
        litems3.push(new_item("-existing-", ""));//include 'libs', global calls, local jumps (since last call, with unmatched ret.)
        //litems3.push(new_item(" label_0x00000001", ""));
        //litems3.push(new_item(" lib_printf(a)", ""));
        litems3
    }

    pub fn get_commands_list(&mut self) -> Vec<ITEM> {
        let mut litems1: Vec<ITEM> = Vec::new();
        litems1.push(new_item("bJMP", "(byte) Jump a, b=cond"));
        litems1.push(new_item("bCLL", "(byte) Call a+b, c=pc"));
        litems1.push(new_item("bADD", "(byte) Add a=b+c"));
        litems1.push(new_item("bSUB", "(byte) Subtract a=b-c"));
        litems1.push(new_item("bBSL", "(byte) Bit-shift left")); 
        litems1.push(new_item("bBSR", "(byte) Bit-shift right"));
        litems1.push(new_item("bRR", "(byte) Rotate right"));
        litems1.push(new_item("bRL", "(byte) Rotate left"));
        litems1.push(new_item("bAND", "(byte) And a=b&c")); 
        litems1.push(new_item("bOR", "(byte) Or a=b|c"));
        litems1.push(new_item("bXOR", "(byte) Xor a=b^c"));   
        litems1.push(new_item("bMUL", "(byte) Multiply a=b*c"));                      
        litems1.push(new_item("bLDR", "(byte) Load a<=[b], inc c"));
        litems1.push(new_item("bSTR", "(byte) Store a=>[b], dec c"));
        litems1.push(new_item("bNOT", "(byte) Not a != b"));
        litems1.push(new_item("bCMP", "(byte) Compare a?b, c=?"));
        litems1.push(new_item("iJMP", "(int) Jump a, b=cond"));
        litems1.push(new_item("iCLL", "(int) Call a+b, c=pc"));
        litems1.push(new_item("iADD", "(int) Add a=b+c"));
        litems1.push(new_item("iSUB", "(int) Subtract a=b-c"));
        litems1.push(new_item("iBSL", "(int) Bit-shift left")); 
        litems1.push(new_item("iBSR", "(int) Bit-shift right"));
        litems1.push(new_item("iRR", "(int) Rotate right"));
        litems1.push(new_item("iRL", "(int) Rotate left"));
        litems1.push(new_item("iAND", "(int) And a=b&c")); 
        litems1.push(new_item("iOR", "(int) Or a=b|c"));
        litems1.push(new_item("iXOR", "(int) Xor a=b^c"));   
        litems1.push(new_item("iMUL", "(int) Multiply a=b*c"));                      
        litems1.push(new_item("iLDR", "(int) Load a=[b], inc c"));
        litems1.push(new_item("iSTR", "(int) Store a=[b], dec c"));
        litems1.push(new_item("iNOT", "(int) Not a != b"));
        litems1.push(new_item("iCMP", "(int) Compare a?b, c=?"));
        litems1
    }
}
