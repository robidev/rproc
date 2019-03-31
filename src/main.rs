#![allow(unused_imports)]
#![allow(dead_code)]
extern crate minifb;
extern crate byteorder;
extern crate num;
extern crate ncurses;
extern crate time;

//#[macro_use]
extern crate enum_primitive;

#[macro_use]
mod utils;
mod virpc;
mod debugger;
mod editor;

use virpc::cpu;
use virpc::memory;

use minifb::*;
use std::env;
use ncurses::*;
use editor::*;

static COLOR_BACKGROUND: i16 = 16;
static COLOR_FOREGROUND: i16 = 17;
static COLOR_KEYWORD: i16 = 18;
static COLOR_PAIR_DEFAULT: i16 = 1;
static COLOR_PAIR_KEYWORD: i16 = 2;

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut prg_to_load  = String::new();
    let mut debugger_on  = false;
    let mut window_scale = Scale::X2;


    // process cmd line params
    for i in 1..args.len() {
        if args[i] == "debugger" {
            debugger_on = true;
        }
        else if args[i] == "x2" {
            window_scale = Scale::X2;
        }
        else if args[i].ends_with(".prg") {
            prg_to_load = args[i].clone();
        }
    }

    if prg_to_load == "" {
        prg_to_load = "test.prg".to_string();
    }

    let mut virpc = virpc::Virpc::new(window_scale, debugger_on, &prg_to_load);
    virpc.reset();
    virpc.run();
    let asmcpu = cpu::CPU::new_shared();
    asmcpu.borrow_mut().set_references(virpc.memory);
    
/*
    // main update loop
    while virpc.main_window.is_open() {
        virpc.run();
    }*/

    initscr();
    keypad(stdscr(), true);
    noecho();
    curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    start_color();
    //init_color(COLOR_BACKGROUND, 0, 0, 0);
    //init_color(COLOR_FOREGROUND, 255, 255, 255);
    init_pair(COLOR_PAIR_DEFAULT, COLOR_WHITE, COLOR_BLACK);
    init_pair(COLOR_PAIR_KEYWORD, COLOR_BLACK, COLOR_WHITE);

    let mut _windows : Windows = Windows::new(asmcpu);

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
    _windows.items1 = litems1;
    Windows::update_menu(_windows.menu1, &mut _windows.items1,0);

    let mut litems2: Vec<ITEM> = Vec::new();
    litems2.push(new_item("register  (0x00)", "1"));//agree on register range
    litems2.push(new_item("new const  (code)", "1"));//byte or int
    litems2.push(new_item("new local  (stack 0x0000FFFF)", "2"));//agree on stack origin (since last call, with unmatched ret.)
    litems2.push(new_item("new global (heap 0x00010000)", "3"));//agree on heap origin
    litems2.push(new_item("-existing-", "4"));
    _windows.items2 = litems2;
    Windows::update_menu(_windows.menu2, &mut _windows.items2,0);

    let mut litems3: Vec<ITEM> = Vec::new();
    litems3.push(new_item("new label", ""));
    litems3.push(new_item("-existing-", ""));//include 'libs', global calls, local jumps (since last call, with unmatched ret.)
    //litems3.push(new_item(" label_0x00000001", ""));
    //litems3.push(new_item(" lib_printf(a)", ""));
    _windows.items3 = litems3;
    Windows::update_menu(_windows.menu3, &mut _windows.items3,0);

    let mut ch = getch();
    while ch != 27 as i32 { // ESC pressed, so quit
        _windows.handle_keys(ch);
        _windows.resize_check();
        ch = getch();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    _windows.destroy();
    //asmcpu.borrow_mut().text_to_instruction("    hello world [test]; aaa    ");
}
