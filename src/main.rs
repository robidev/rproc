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
    init_color(COLOR_BACKGROUND, 0, 43 * 4, 54 * 4);
    init_color(COLOR_FOREGROUND, 142 * 4, 161 * 4, 161 * 4);
    init_pair(COLOR_PAIR_DEFAULT, COLOR_FOREGROUND, COLOR_BACKGROUND);
    init_pair(COLOR_PAIR_KEYWORD, COLOR_KEYWORD, COLOR_BACKGROUND);

    let mut _windows : Windows = Windows::new();

    let mut pc = 0;
    for _i in 0..4 {
        pc = asmcpu.borrow_mut().disassemble(pc);
        _windows.wprintw_pad(asmcpu.borrow_mut().instruction_to_text().as_str());
    }

    let mut ch = getch();
    while ch != 27 as i32 { // ESC pressed, so quit
        _windows.handle_keys(ch);
        _windows.resize_check();
        ch = getch();
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    _windows.destroy();
    asmcpu.borrow_mut().text_to_instruction("    hello world [test]; aaa    ");
}
