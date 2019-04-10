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

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut prg_to_load  = "test.prg".to_string();
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
    
    let mut virpc = virpc::Virpc::new(window_scale, debugger_on, &prg_to_load);

    let asmcpu = cpu::CPU::new_shared();
    virpc.reset();

    asmcpu.borrow_mut().set_references(virpc.memory);
    let mut _windows : Windows = Windows::new(asmcpu);

    let mut ch = getch();
    while ch != 27 as i32 { // ESC pressed, so quit
        _windows.handle_keys(ch);
        _windows.resize_check();
        ch = getch();

        //virpc.run();

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    _windows.destroy();

    //asmcpu.borrow_mut().text_to_instruction("    hello world [test]; aaa    ");

}

