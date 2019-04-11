#![allow(non_snake_case)]
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
pub const CODE: u32 = 0x00000001;
pub const BSS: u32 = 0x0000E000;
pub const REGISTERS: u32 = 0x0000F000;//til 0x0000FFFF
pub const MEMORY: u32 = 0x00010000;
pub const STACK: u32 = 0x00080000;

pub const PC_REG: u32 = 0xF000;
pub const STACK_REG: u32 = 0xF001;
pub const A_REG: u32 = 0xF002;
pub const B_REG: u32 = 0xF003;
pub const C_REG: u32 = 0xF004;

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

pub struct Items {
    name : String,
    description : String,
    value : u32,
}

pub struct CPU {
    pub pc: u32, // program counter
    pub p:  u8,  // processor status
    pub mem_ref:  Option<memory::MemShared>, // reference to shared system memory

    pub instruction: opcodes::Instruction,
    pub instruction_u8 : u8,
    pub state: CPUState,
    pub prev_pc: u32,
    pub data : Vec<Items>,
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
            data : CPU::get_variables_list(),
        }))
    }

    pub fn destroy(&mut self) {
        self.data.clear();
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
                        self.instruction_u8 = next_op;
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

    pub fn load_opcode_data(&mut self, address: u32) {
        self.pc = address; //retrieve next byte
        self.prev_pc = self.pc;
        let next_op = self.next_byte();
        match opcodes::get_instruction(next_op) { //retrieve instruction
            Some((opcode, size, arguments, addr_type)) => {
                self.instruction.opcode = opcode;
                self.instruction.size = size;
                self.instruction.args = arguments;
                self.instruction.addressing_type = addr_type;
                self.instruction_u8 = next_op;
            }
            None => panic!("Can't fetch instruction")
        }
        opcodes::pull_operand_addr(self);
    }

    pub fn disassemble(&mut self, address: u32) -> u32 {
        //take a memory address
        //return the opcode and arguments
        self.load_opcode_data(address);

        //parse arguments 
        match self.instruction.opcode {
            /*Op::LDR => {// LDR if pc-rel,pop, add to local
                let d = " ".to_string();
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
                                    //pop
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
            },*/
            // STR if pc-rel,push remove from local
            Op::STR => {
                let mut index = 0;
                let d = " ".to_string();
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
                        index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[1]) ); 
                        self.instruction.arg_index[1] = index;//index of arg in list                       
                    }
                    else {//arg2+const
                        if self.instruction.args & 0x01 == 0 {//push 
                            if self.instruction.arg[2] == STACK_REG {
                                // add arg1 local var
                                 let s = format!("LOCALVAR_{:08X}",self.instruction.arg[1]+self.read_int_le(self.instruction.arg[2])); 
                                 index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[1]) );
                                 self.instruction.arg_index[2] = index;//index of arg in list
                            }
                            else {
                                if self.instruction.arg[2] > BSS {
                                    // add arg2 to global var /reg
                                    let s = format!("VAR_{:08X}",self.instruction.arg[1]+self.read_int_le(self.instruction.arg[2])); 
                                    index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[1]) );
                                    self.instruction.arg_index[2] = index;//index of arg in list
                                }
                                else {
                                    // add arg2 to global const 
                                    let s = format!("CONST_{:08X}",self.instruction.arg[1]+self.read_int_le(self.instruction.arg[2])); 
                                    index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[1]) );
                                    self.instruction.arg_index[2] = index;//index of arg in list
                                }
                            }                               
                        }
                        else {//str([arg2+sp]=arg0)
                            //value from VAR[arg1+[arg2]]
                            let s = format!("LOCALVAR_{:08X}",self.instruction.arg[1]+self.instruction.arg[2]); 
                            index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[1]) );
                            self.instruction.arg_index[2] = index;//index of arg in list
                        }
                    }
                }
            },
            // CALL: add if not exist addr(pc-rel, or static) to label-list
            Op::CLL => {
                let d = " ".to_string();
                if self.instruction.args & 0x04 == 0 && self.instruction.args & 0x02 == 0 {
                    let s = format!("LABEL_{:08X}",self.instruction.arg[0] + self.instruction.arg[1]);
                    let index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[0]) );
                    self.instruction.arg_index[0] = index;//index of arg in list
                }
            },
            // JMP, add if not exist addr(pc-rel, or static) to label-list
            Op::JMP => {
                let d = " ".to_string();
                let mut s;
                if self.instruction.args & 0x02 == 0 { s = format!("CONST_{:08X}",self.instruction.arg[1]); }
                else { s = format!("REF_{:08X}",self.instruction.arg[1]); }

                self.instruction.arg_index[1] = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d.clone(),self.instruction.arg[1]) );

                if self.instruction.args & 0x04 == 0 {
                    if self.instruction.arg_index[1] < 9 {
                        let s   = format!("LABEL_{:08X}",self.instruction.arg[0]);
                        let index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d, self.instruction.arg[0]) );
                        self.instruction.arg_index[0] = index;//index of arg in list
                    }
                    else {
                        let s = format!("LABEL_{:08X} (pc+{})",self.instruction.arg[0] + self.pc, self.instruction.arg[0]);
                        let index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[0]) );
                        self.instruction.arg_index[0] = index;//index of arg in list
                    }
                } 
                else {
                    let s = format!("REF_[{:08X}]",self.instruction.arg[0]);
                    let index = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[0]) );
                    self.instruction.arg_index[0] = index;//index of arg in list
                }
            },
            _ => {//any other opcode, default behaviour
                for i in 0..(self.instruction.size as usize) {
                    let d = " ".to_string();
                    if (self.instruction.args << i) & 0x04 > 0 || (i == 0 && self.instruction.args & 0x04 == 0) {//arg 0 is always a ref, subsequent are const or ref
                        let mut s = "".to_string();
                        let mut v = 0;
                        match self.instruction.arg[i] {
                            RESET_VECTOR => { s = "$RESET".to_string(); },
                            CODE...BSS => { s = format!("LABEL_{:08X}",self.instruction.arg[i]); v=self.instruction.arg[i]; },
                            BSS...REGISTERS => { s = format!("REG_{:08X}",self.instruction.arg[i]); v=self.instruction.arg[i]; },
                            REGISTERS...MEMORY => { s = format!("BSS_{:08X}",self.instruction.arg[i]); v=self.instruction.arg[i]; },
                            MEMORY...STACK => { s = format!("VAR_{:08X}",self.instruction.arg[0]); v=self.instruction.arg[0]; },//heap
                            _ => {},
                        }
                        self.instruction.arg_index[i] = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d, v) );
                    }
                    else {
                        let mut s;
                        if i == 0 {
                            s = format!("REF_[{:08X}]",self.instruction.arg[i]);
                        } else {
                            s = format!("CONST_{:08X}",self.instruction.arg[i]);
                        }
                        
                        self.instruction.arg_index[i] = CPU::add_new_item(&mut self.data, CPU::new_Item(s, d,self.instruction.arg[i]) );                       
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

    pub fn add_new_item(items : &mut Vec<Items>, new_item : Items) -> u32 {
        for i in 0..items.len() {
            if items[i].name == new_item.name {
                if items[i].description == new_item.description {
                    return i as u32;
                }
            }
        }
        items.push(new_item);
        (items.len()-1) as u32
    }

    pub fn new_Item (n : String, d : String, v : u32) -> Items {
        Items {
            name : n,
            description : d,
            value : v,
        }
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

    pub fn argument_type(&mut self, arg : u32) -> u32 {
        let cur_arg = (self.instruction_u8 & 0x0F) << arg;
        if cur_arg & 0x04 == 0 { 0 }
        else { 1 }
    }

    pub fn get_data_list(&mut self) ->  Vec<ITEM> {
        let mut litems_d: Vec<ITEM> = Vec::new();
        for it in self.data.iter() {
            litems_d.push(new_item(it.name.as_bytes() , it.description.as_bytes() ));
        }
        litems_d
    }

    pub fn get_variables_list() -> Vec<Items> {
        //include 'libs', global calls, local jumps (since last call, with unmatched ret.)
        let mut litems: Vec<Items> = Vec::new();
        litems.push(CPU::new_Item("register  (0xF000)".to_string(), " ".to_string(),0));//agree on register range
        litems.push(CPU::new_Item("new const  (code)".to_string(), " ".to_string(),0));//byte or int
        litems.push(CPU::new_Item("new local  (stack 0x0000FFFF)".to_string(), " ".to_string(),0));//agree on stack origin (since last call, with unmatched ret.)
        litems.push(CPU::new_Item("new global (heap 0x00010000)".to_string(), " ".to_string(),0));//agree on heap origin
        litems.push(CPU::new_Item("-existing-".to_string(), " ".to_string(),0));
        litems
    }

    pub fn get_addressing_mode_list(&mut self) -> Vec<ITEM> {
        let mut litems3: Vec<ITEM> = Vec::new();
        litems3.push(new_item("direct".as_bytes(), " ".as_bytes()));
        litems3.push(new_item("indirect".as_bytes(), " ".as_bytes()));
        litems3
    }

    pub fn get_commands_list(&mut self) -> Vec<ITEM> {
        let mut litems1: Vec<ITEM> = Vec::new();
        litems1.push(new_item("bJMP".as_bytes(), "(byte) Jump a, b=cond".as_bytes()));
        litems1.push(new_item("bCLL".as_bytes(), "(byte) Call a+b, c=pc".as_bytes()));
        litems1.push(new_item("bADD".as_bytes(), "(byte) Add a=b+c".as_bytes()));
        litems1.push(new_item("bSUB".as_bytes(), "(byte) Subtract a=b-c".as_bytes()));
        litems1.push(new_item("bBSL".as_bytes(), "(byte) Bit-shift left".as_bytes())); 
        litems1.push(new_item("bBSR".as_bytes(), "(byte) Bit-shift right".as_bytes()));
        litems1.push(new_item("bRR".as_bytes(), "(byte) Rotate right".as_bytes()));
        litems1.push(new_item("bRL".as_bytes(), "(byte) Rotate left".as_bytes()));
        litems1.push(new_item("bAND".as_bytes(), "(byte) And a=b&c".as_bytes())); 
        litems1.push(new_item("bOR".as_bytes(), "(byte) Or a=b|c".as_bytes()));
        litems1.push(new_item("bXOR".as_bytes(), "(byte) Xor a=b^c".as_bytes()));   
        litems1.push(new_item("bMUL".as_bytes(), "(byte) Multiply a=b*c".as_bytes()));                      
        litems1.push(new_item("bLDR".as_bytes(), "(byte) Load a<=[b], inc c".as_bytes()));
        litems1.push(new_item("bSTR".as_bytes(), "(byte) Store a=>[b], dec c".as_bytes()));
        litems1.push(new_item("bNOT".as_bytes(), "(byte) Not a != b".as_bytes()));
        litems1.push(new_item("bCMP".as_bytes(), "(byte) Compare a?b, c=?".as_bytes()));
        litems1.push(new_item("iJMP".as_bytes(), "(int) Jump a, b=cond".as_bytes()));
        litems1.push(new_item("iCLL".as_bytes(), "(int) Call a+b, c=pc".as_bytes()));
        litems1.push(new_item("iADD".as_bytes(), "(int) Add a=b+c".as_bytes()));
        litems1.push(new_item("iSUB".as_bytes(), "(int) Subtract a=b-c".as_bytes()));
        litems1.push(new_item("iBSL".as_bytes(), "(int) Bit-shift left".as_bytes())); 
        litems1.push(new_item("iBSR".as_bytes(), "(int) Bit-shift right".as_bytes()));
        litems1.push(new_item("iRR".as_bytes(), "(int) Rotate right".as_bytes()));
        litems1.push(new_item("iRL".as_bytes(), "(int) Rotate left".as_bytes()));
        litems1.push(new_item("iAND".as_bytes(), "(int) And a=b&c".as_bytes())); 
        litems1.push(new_item("iOR".as_bytes(), "(int) Or a=b|c".as_bytes()));
        litems1.push(new_item("iXOR".as_bytes(), "(int) Xor a=b^c".as_bytes()));   
        litems1.push(new_item("iMUL".as_bytes(), "(int) Multiply a=b*c".as_bytes()));                      
        litems1.push(new_item("iLDR".as_bytes(), "(int) Load a=[b], inc c".as_bytes()));
        litems1.push(new_item("iSTR".as_bytes(), "(int) Store a=[b], dec c".as_bytes()));
        litems1.push(new_item("iNOT".as_bytes(), "(int) Not a != b".as_bytes()));
        litems1.push(new_item("iCMP".as_bytes(), "(int) Compare a?b, c=?".as_bytes()));
        litems1
    }

    pub fn set_opcode(&mut self, cmd : i32, mod1 : i32, mod2 : i32, mod3 : i32) {
        //-1 means not set, so dont modify
        let mut code : u8;
        
        if cmd ==-1 { code = self.instruction_u8 & 0xF8; }
        else { 
            code = (((cmd &0x0F) << 4) & 0xF0) as u8;
            if cmd & 0x10 > 0 { code |= 0x08; }
        }

        if mod1 ==-1 { code |= self.instruction.args & 0x04; }
        else { if mod1 == 1  { code |= 0x04; } }

        if mod2 ==-1 { code |= self.instruction.args & 0x02; }
        else { if mod2 == 1  { code |= 0x02; } }

        if mod3 ==-1 { code |= self.instruction.args & 0x01; }
        else { if mod3 == 1  { code |= 0x01; } }

        match opcodes::get_instruction(code) { //retrieve instruction
            Some((opcode, size, arguments, addr_type)) => {
                self.instruction.opcode = opcode;
                self.instruction.size = size;
                self.instruction.args = arguments;
                self.instruction.addressing_type = addr_type;
                self.instruction_u8 = code;
            }
            None => panic!("Can't fetch instruction")
        }
    }

    pub fn parse_args(&mut self, arg1 : i32, arg2 : i32, arg3 : i32) {
        //-1 means not set, check with self.data
        if arg1 > 4  && arg1 < self.data.len() as i32 { 
            self.instruction.arg[0] = self.data[arg1 as usize].value;
        }
        if arg2 > 4  && arg2 < self.data.len() as i32 { 
            self.instruction.arg[1] = self.data[arg2 as usize].value;
        }
        if arg3 > 4  && arg3 < self.data.len() as i32 { 
            self.instruction.arg[2] = self.data[arg3 as usize].value;
        }
    }
}
