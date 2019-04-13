// opcode enumeration suffix: // addressing mode:
// imm = #$00                 // immediate 
// zp = $00                   // zero page
// zpx = $00,X                // zero page with X
// zpy = $00,Y                // zero page with Y
// izx = ($00,X)              // indexed indirect (X)
// izy = ($00),Y              // indirect indexed (Y)
// abs = $0000                // absolute
// abx = $0000,X              // absolute indexed with X
// aby = $0000,Y              // absolute indexed with Y
// ind = ($0000)              // indirect
// rel = $0000                // relative to PC/IP

use crate::virpc::cpu;
use std::fmt;

pub enum ArgumentSize {
    Byte,
    Int,
}

pub enum Op {
    // Load/store
    LDR, STR,
    // Logical
    AND, OR, XOR, NOT,
    // Arithmetic
    ADD, SUB,
    MUL,
    // Shifts
    RR, RL,
    BSL, BSR,
    // Jump calls
    JMP,
    // Branches
    CLL,
    // Status flag changes
    CMP,
    // System functions
}

pub struct Instruction {
    pub opcode: Op,
    pub size: u8,  // arguments, max 3
    pub args: u8,  // immediate, or reference, for each argument
    pub addressing_type: ArgumentSize, //byte or int
    pub arg:Vec<u32>,
    //menu index
    pub arg_index:Vec<u32>,
}

impl Instruction {
    pub fn new() -> Instruction {
        let mut instruction = Instruction {
            opcode: Op::CMP,
            size: 0,
            args: 0,
            addressing_type: ArgumentSize::Int,
            arg: Vec::<u32>::new(),
            arg_index: Vec::<u32>::new(),
        };
        for _ in 0..3 {
            instruction.arg.push(0);
            instruction.arg_index.push(0);
        }
        instruction
    }
}

// debug display for opcodes
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let op_name = match self.opcode {
            Op::LDR => "LDR", Op::STR => "STR",
            Op::AND => "AND", Op::OR => "OR", Op::XOR => "XOR", Op::NOT => "NOT",
            Op::ADD => "ADD", Op::SUB => "SUB",
            Op::MUL => "MUL", 
            Op::RR => "RR", Op::RL => "RL", 
            Op::BSL => "BSL", Op::BSR => "BSR",
            Op::JMP => "JMP", 
            Op::CLL => "CLL", 
            Op::CMP => "CMP",
        };
        
        write!(f, "{}", op_name)
    }
}


// runs the instruction
pub fn run(cpu: &mut cpu::CPU) -> bool {
    match cpu.instruction.opcode {
        Op::JMP => { 
            match cpu.instruction.arg[1] {
                
                1 => { if cpu.get_status_flag(cpu::StatusFlag::Carry) { cpu.pc = cpu.instruction.arg[0]; } },
                2 => { if cpu.get_status_flag(cpu::StatusFlag::Zero ) { cpu.pc = cpu.instruction.arg[0]; } },
                3 => { if cpu.get_status_flag(cpu::StatusFlag::Overflow) { cpu.pc = cpu.instruction.arg[0]; } },
                4 => { if cpu.get_status_flag(cpu::StatusFlag::Negative) { cpu.pc = cpu.instruction.arg[0]; } },

                5 => { if !cpu.get_status_flag(cpu::StatusFlag::Carry) { cpu.pc = cpu.instruction.arg[0]; } },
                6 => { if !cpu.get_status_flag(cpu::StatusFlag::Zero ) { cpu.pc = cpu.instruction.arg[0]; } },
                7 => { if !cpu.get_status_flag(cpu::StatusFlag::Overflow) { cpu.pc = cpu.instruction.arg[0]; } },
                8 => { if !cpu.get_status_flag(cpu::StatusFlag::Negative) { cpu.pc = cpu.instruction.arg[0]; } },

                9 => { if cpu.get_status_flag(cpu::StatusFlag::Carry) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                10 => { if cpu.get_status_flag(cpu::StatusFlag::Zero ) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                11 => { if cpu.get_status_flag(cpu::StatusFlag::Overflow) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                12 => { if cpu.get_status_flag(cpu::StatusFlag::Negative) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },

                13 => { if !cpu.get_status_flag(cpu::StatusFlag::Carry) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                14 => { if !cpu.get_status_flag(cpu::StatusFlag::Zero ) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                15 => { if !cpu.get_status_flag(cpu::StatusFlag::Overflow) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },
                16 => { if !cpu.get_status_flag(cpu::StatusFlag::Negative) { cpu.pc = (cpu.prev_pc as i32 + cpu.instruction.arg[0] as i32) as u32; } },

                17 => { cpu.pc = (cpu.pc as i32 + cpu.instruction.arg[0] as i32) as u32; },
                _ => cpu.pc = cpu.instruction.arg[0],
            }
        },
        Op::CLL => {
            let pos = cpu.pc;
            cpu.pc = cpu.instruction.arg[0]+cpu.instruction.arg[1];

            //call A+B, and store pos on C
            let adr = cpu.instruction.arg[2];
            cpu.write_int_le(adr,pos);
        },
        Op::LDR => {//LDR/POP 3 POP A from [B] (and inc c) TODO: add pc relative addressing, and sp relative addressing
            match cpu.instruction.addressing_type { 
                ArgumentSize::Byte => {
                    //read val from [arg1], store in arg0 (and inc addr2)
                    if cpu.instruction.args & 0x02 > 0 {
                        let val = cpu.instruction.arg[1];
                        cpu.write_byte(cpu.instruction.arg[0],val as u8);

                        //increment value that c points to, if args==xx0
                        if cpu.instruction.args & 0x01 == 0 {
                            let stack = cpu.read_int_le(cpu.instruction.arg[2]);
                            cpu.write_int_le(cpu.instruction.arg[2], stack + 1);
                        }
                    }
                    else {//if 2nd arg is not a reference, special case: pc relative ldr, or arg2 relative
                        //read val from pc+const_arg1, store in arg0
                        if cpu.instruction.arg[2] == 0 {
                            let val = cpu.read_byte((cpu.instruction.arg[1] as i32 + cpu.pc as i32) as u32);
                            cpu.write_byte(cpu.instruction.arg[0],val);   
                        }
                        //arg2 is valid, so use as stack-pointer
                        else {
                            //read val from [addr](+const), and inc addr => pop a/[a]
                            if cpu.instruction.args & 0x01 == 0 {//pop(a=[[stack+b]++]) = ldr 1 b010,
                                let stack = cpu.read_int_le(cpu.instruction.arg[2]);
                                let val = cpu.read_byte((cpu.instruction.arg[1] as i32 + stack as i32) as u32);
                                cpu.write_byte(cpu.instruction.arg[0],val);  
                                cpu.write_int_le(cpu.instruction.arg[2], stack + 1);                           
                            }
                            //read val from [addr]+const
                            else {//ldr(a=[b+sp])
                                let stack = cpu.instruction.arg[2];
                                let val = cpu.read_byte((cpu.instruction.arg[1] as i32 + stack as i32) as u32);
                                cpu.write_byte(cpu.instruction.arg[0],val);                                  
                            }

                        }                     
                    }
                }
                ArgumentSize::Int => {  //ldr(a=[b]), optional: [c]++
                    //read val from [arg1], store in arg0 (and inc addr2)
                    if cpu.instruction.args & 0x02 > 0 {//if arg1 is ref
                        let val = cpu.instruction.arg[1];//value from ref(A/B/C)
                        cpu.write_int_le(cpu.instruction.arg[0],val);//arg0/[arg0] = value

                        //increment value that c points to, if args==xx0
                        if cpu.instruction.args & 0x01 == 0 {
                            let stack = cpu.read_int_le(cpu.instruction.arg[2]);//arg2=1=>500, stack=500
                            cpu.write_int_le(cpu.instruction.arg[2], stack + 4);//1<-504
                        }
                    }
                    else {//if 2nd arg is not a reference, special case: pc relative, or arg2 relative ldr
                        //read val from pc+const_arg1, store in arg0
                        if cpu.instruction.arg[2] == 0 {//ldr(a=[b+pc])
                            let val = cpu.read_int_le((cpu.instruction.arg[1] as i32 + cpu.pc as i32) as u32);//value from [pc+arg1]
                            cpu.write_int_le(cpu.instruction.arg[0],val);//arg0/[arg0] = value
                        } 
                        //arg2 is valid, so use as stack-pointer
                        else {
                            //read val from [addr](+const), and inc addr => pop a/[a]
                            //increment value that c points to, if args==xx0
                            if cpu.instruction.args & 0x01 == 0 {//pop(a=[[stack+b]++]) = ldr 1 b010,
                                let stack = cpu.read_int_le(cpu.instruction.arg[2]);//arg2=1=>500, stack=500 
                                let val = cpu.read_int_le((cpu.instruction.arg[1] as i32 + stack as i32) as u32);//value from [arg1+stack]
                                cpu.write_int_le(cpu.instruction.arg[0],val);//arg0/[arg0] = value
                                cpu.write_int_le(cpu.instruction.arg[2], stack + 4);//1<-504
                            }
                            //read val from [addr]+const
                            else {//ldr(a=[b+sp])
                                let stack = cpu.instruction.arg[2];//arg2=1=>500, stack=500 
                                let val = cpu.read_int_le((cpu.instruction.arg[1] as i32 + stack as i32) as u32);//value from [arg1+stack]
                                cpu.write_int_le(cpu.instruction.arg[0],val);//arg0/[arg0] = value
                            }
                        }                  
                    }
                }
            };
        },
        Op::STR => {//STR/PUSH 3 PUSH A on [B] (and inc c) 
            match cpu.instruction.addressing_type { 
                ArgumentSize::Byte => {
                    if cpu.instruction.args & 0x02 > 0 {
                        let adr = cpu.instruction.arg[1];
                        cpu.write_byte(adr,cpu.instruction.arg[0] as u8);//[arg1]=arg0/[arg0]

                        if cpu.instruction.args & 0x01 == 0 {//inc arg2
                            let stack = cpu.read_int_le(cpu.instruction.arg[2]);
                            cpu.write_int_le(cpu.instruction.arg[2], stack - 1);                      
                        }
                    }
                    else {//if 2nd arg is not a reference, special case: pc relative str, or arg2 relative
                        if cpu.instruction.arg[2] == 0 {
                            let adr = (cpu.instruction.arg[1] as i32 + cpu.pc as i32) as u32;
                            cpu.write_byte(adr,cpu.instruction.arg[0] as u8);
                        }
                        else {
                            if cpu.instruction.args & 0x01 == 0 {
                                let stack = cpu.read_int_le(cpu.instruction.arg[2]);//arg2=1, stack = 500
                                let adr = (cpu.instruction.arg[1] as i32 + stack as i32) as u32;//adr = 500+arg1
                                cpu.write_byte(adr,cpu.instruction.arg[0] as u8);  //[adr] = arg0/[arg0]
                                cpu.write_int_le(cpu.instruction.arg[2], stack - 1); //1<-501
                            }
                            else {
                                let stack = cpu.instruction.arg[2];
                                let adr = (cpu.instruction.arg[1] as i32 + stack as i32) as u32;
                                cpu.write_byte(adr,cpu.instruction.arg[0] as u8);                                
                            }
                        }
                    }
                }
                ArgumentSize::Int => {
                    if cpu.instruction.args & 0x02 > 0 {
                        let adr = cpu.instruction.arg[1];
                        cpu.write_int_le(adr,cpu.instruction.arg[0]);
                        //if c is not 0, decrement value that c points to
                        if cpu.instruction.args & 0x01 == 0 {
                            let stack = cpu.read_int_le(cpu.instruction.arg[2]);
                            cpu.write_int_le(cpu.instruction.arg[2], stack - 4);
                        }
                    }
                    else {//if 1st arg is not a reference, special case: pc relative str, or arg2 relative
                        if cpu.instruction.arg[2] == 0 {//PC relative
                            let adr = (cpu.instruction.arg[1] as i32 + cpu.pc as i32) as u32;
                            cpu.write_int_le(adr,cpu.instruction.arg[0]);
                        }
                        else {//arg2+const
                            if cpu.instruction.args & 0x01 == 0 {
                                let stack = cpu.read_int_le(cpu.instruction.arg[2]);
                                let adr = (cpu.instruction.arg[1] as i32 + stack as i32) as u32;
                                cpu.write_int_le(adr,cpu.instruction.arg[0]);  
                                cpu.write_int_le(cpu.instruction.arg[2], stack - 4);                               
                            }
                            else {
                                let stack = cpu.instruction.arg[2];
                                let adr = (cpu.instruction.arg[1] as i32 + stack as i32) as u32;
                                cpu.write_int_le(adr,cpu.instruction.arg[0]);                                
                            }
                        }
                    }
                }
            };
        },
        Op::CMP => {
            //set cpsr regarding A and B
            match cpu.instruction.addressing_type { 
                ArgumentSize::Byte => {     
                    let v = cpu.instruction.arg[0];
                    let res = cpu.instruction.arg[1] as i16 - v as i16;
                    cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
                    cpu.set_zn_flags(res as u8);
                }
                ArgumentSize::Int => {
                    let v = cpu.instruction.arg[0];
                    let res = cpu.instruction.arg[1] as i64 - v as i64;
                    cpu.set_status_flag(cpu::StatusFlag::Carry, res >= 0);
                    cpu.set_zn_flags_int(res as u32);                    
                }
            };
        },
        Op::RR => {
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            match cpu.instruction.addressing_type { 
                ArgumentSize::Byte => {           
                    let v = cpu.instruction.arg[1] as u8;
                    cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x01) != 0);
                    let mut res = v >> 1;
                    if c {
                        res |= 0x80;
                    }
                    cpu.write_byte(cpu.instruction.arg[0],res);
                    cpu.set_zn_flags(res);
                }
                ArgumentSize::Int => {
                    let v = cpu.instruction.arg[1];
                    cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x00000001) != 0);
                    let mut res = v >> 1;
                    if c {
                        res |= 0x80000000;
                    }
                    cpu.write_int_le(cpu.instruction.arg[0],res);
                    cpu.set_zn_flags_int(res);
                }
            };

        },
        Op::RL => {
            let c = cpu.get_status_flag(cpu::StatusFlag::Carry);
            match cpu.instruction.addressing_type { 
                ArgumentSize::Byte => {  
                    let v = cpu.instruction.arg[1] as u8;
                    cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80) != 0);
                    let mut res = v << 1;
                    if c {
                        res |= 0x01;
                    }
                    cpu.write_byte(cpu.instruction.arg[0],res);
                    cpu.set_zn_flags(res);
                }
                ArgumentSize::Int => {
                    let v = cpu.instruction.arg[1];
                    cpu.set_status_flag(cpu::StatusFlag::Carry, (v & 0x80000000) != 0);
                    let mut res = v << 1;
                    if c {
                        res |= 0x00000001;
                    }
                    cpu.write_int_le(cpu.instruction.arg[0],res);
                    cpu.set_zn_flags_int(res);
                }                
            };
        },
        Op::AND => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] & cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] & cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::OR => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] | cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] | cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::XOR => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] ^ cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] ^ cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::NOT => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1]^0; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1]^0; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::ADD => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] + cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] + cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::SUB => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] - cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] - cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::MUL => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] * cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] * cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::BSL => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] << cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] << cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        Op::BSR => {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => {
                    let result = cpu.instruction.arg[1] >> cpu.instruction.arg[2]; 
                    cpu.write_byte(cpu.instruction.arg[0], result as u8); 
                }
                ArgumentSize::Int => {
                    let result = cpu.instruction.arg[1] >> cpu.instruction.arg[2]; 
                    cpu.write_int_le(cpu.instruction.arg[0], result); 
                }
            };
        },
        //_ => panic!("Unknown instruction: {} at ${:04X}", cpu.instruction, cpu.pc)
    }
    // instruction finished execution?
    true
}

pub fn fetch_operand_addr(cpu: &mut cpu::CPU) -> bool {
    for arg_i in 0..cpu.instruction.size {
        if (cpu.instruction.args << arg_i) & 0x04 > 0 {
            let ref_val = cpu.next_int();
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => { cpu.instruction.arg[arg_i as usize] = cpu.read_byte(ref_val) as u32; },
                ArgumentSize::Int => {  cpu.instruction.arg[arg_i as usize] = cpu.read_int_le(ref_val); },
            }
        }
        else {
            match cpu.instruction.addressing_type {
                ArgumentSize::Byte => { cpu.instruction.arg[arg_i as usize] = cpu.next_byte() as u32; },
                ArgumentSize::Int => { cpu.instruction.arg[arg_i as usize] = cpu.next_int(); },
            }
        }
    }
    true
}

// num cycles represents the *max* number of cycles that the instruction can take to execute
// (so taking into account extra cycles for branching, page crosses etc.)
pub fn get_instruction(opcode: u8) -> Option<(Op, u8, u8, ArgumentSize)> {

    //TODO make this into an enum
    let args:u8 = opcode & 0x07;//max 3 arguments, each can be immediate or a memmory-address to a value or pointer based on the context

    let addr_type:ArgumentSize;

    if opcode & 0x08 > 0 {
        addr_type = ArgumentSize::Int;
    }
    else{
        addr_type = ArgumentSize::Byte
    }
    
    //jmp b = options

    //  arg0 can be arg0/[arg0]
    //
    //ldr   arg0 = [arg1], (with inc arg2),         (arg1==[], arg2=*)
    //      arg0 = [pc+arg1(const)] (arg2=0)        (arg1!=[], arg2==0)
    //      pop from [arg1(const)+arg2++] into arg0 (arg1!=[], arg2!=[]) 
    //      arg0 = [addr+const]                     (arg1!=[], arg2==[])

    //str   [arg1] = arg0, (with inc arg2),         (arg1==[], arg2=*)
    //      [pc+arg1(const)] = arg0 (arg2=0)        (arg1!=[], arg2==0)
    //      push arg0 onto [arg1(const) + arg2++]   (arg1!=[], arg2!=[])
    //      [arg1(const) + arg2] = arg0             (arg1!=[], arg2==[])
    Some(match opcode & 0xF0 {                  
        /*JMP      */ 0x00 => (Op::JMP, 2,args,addr_type), //- A, B=modifiers EQ/NE Z/NZ C/NC V/NV Q/NQ GE/LE G/L (4 bits)
        /*CALL     */ 0x10 => (Op::CLL, 3,args,addr_type), // CALL 3 CALL A+B [C]=pos
        //Byte/Int (1 bit), immediate, address(1bit*3 arg)
        /*ADD      */ 0x20 => (Op::ADD, 3,args,addr_type), // ADD 3 A=B+C (can also be MOV)
        /*SUB      */ 0x30 => (Op::SUB, 3,args,addr_type),   // SUB 3 A=B-C
        /*BSL      */ 0x40 => (Op::BSL, 3,args,addr_type), // BSL 3 A=B<<C
        /*BSR      */ 0x50 => (Op::BSR, 3,args,addr_type), // BSR 3 A=B>>C
        /*RR       */ 0x60 => (Op::RR, 3,args,addr_type), // RR 3 A=B RR C times
        /*RL       */ 0x70 => (Op::RL, 3,args,addr_type), // RL 3 A=B RL C times
        /*AND      */ 0x80 => (Op::AND, 3,args,addr_type), // AND 3 A = B AND C
        /*OR       */ 0x90 => (Op::OR, 3,args,addr_type), // OR  3 A = B OR C
        /*XOR      */ 0xA0 => (Op::XOR, 3,args,addr_type), // XOR 3 A=B^C
        /*MUL      */ 0xB0 => (Op::MUL, 3,args,addr_type), // MUL 3 A=B*C
        /*STR      */ 0xC0 => (Op::STR, 3,args,addr_type), // STR/PUSH 3 PUSH A on [B] (and dec c)
        //if 2nd arg is not a reference, special case: pc relative str, or arg2 relative

        /*LDR      */ 0xD0 => (Op::LDR, 3,args,addr_type), // LDR/POP 3 POP A from [B] (and inc c)
        //if 2nd arg is not a reference, special case: pc relative ldr, or arg2 relative
        
        /*NOT      */ 0xE0 => (Op::NOT, 2,args,addr_type), // NOT 2 A!=B (c=OPTIONS?)
        /*CMP      */ 0xF0 => (Op::CMP, 2,args,addr_type), // CMP 2 A?B (c=OPTIONS?)
                         _ => return None
    })
}

pub fn get_opcode(cpu: &mut cpu::CPU) -> u8 {
    let mut op_val = match cpu.instruction.opcode {
            Op::JMP => 0x00,
            Op::CLL => 0x10,
            Op::ADD => 0x20, 
            Op::SUB => 0x30,
            Op::BSL => 0x40, 
            Op::BSR => 0x50,
            Op::RR => 0x60, 
            Op::RL => 0x70, 
            Op::AND => 0x80, 
            Op::OR => 0x90, 
            Op::XOR => 0xA0,   
            Op::MUL => 0xB0,                       
            Op::LDR => 0xC0, 
            Op::STR => 0xD0,
            Op::NOT => 0xE0,
            Op::CMP => 0xF0,
        };
        match cpu.instruction.addressing_type {
            ArgumentSize::Int => { op_val |= 0x08 },
            _ => {}
        }
        op_val |= cpu.instruction.args & 0x07;
        op_val
}

pub fn push_operand_addr(cpu: &mut cpu::CPU) -> bool {
    for arg_i in 0..cpu.instruction.size {
        match cpu.instruction.addressing_type {
            ArgumentSize::Int => { cpu.write_int_le(cpu.pc,cpu.instruction.arg[arg_i as usize] ); cpu.pc += 4; },
            ArgumentSize::Byte => { cpu.write_byte(cpu.pc,cpu.instruction.arg[arg_i as usize] as u8); cpu.pc += 1; },//TODO, is this a value(byte), or an address?(int)
        }
    }
    true
}

pub fn pull_operand_addr(cpu: &mut cpu::CPU) -> bool {
    for arg_i in 0..cpu.instruction.size {
        match cpu.instruction.addressing_type {
            ArgumentSize::Int => cpu.instruction.arg[arg_i as usize] = cpu.next_int(),
            ArgumentSize::Byte => cpu.instruction.arg[arg_i as usize] = cpu.next_byte() as u32,
        }
    }
    true
}