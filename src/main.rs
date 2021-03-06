//#![allow(unused_imports)]
//#![allow(dead_code)]
extern crate minifb;
extern crate byteorder;
extern crate num;
extern crate ncurses;
extern crate time;
extern crate enum_primitive;



#[macro_use]
mod utils;
mod virpc;
mod debugger;
mod editor;

use virpc::cpu;
use minifb::*;
use std::env;
use ncurses::*;
use editor::*;

use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicIsize, Ordering};

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

    let asmcpu = cpu::CPU::new_shared(0xFF00);
    virpc.reset();
    virpc.run();
    asmcpu.borrow_mut().set_references(virpc.memory.clone());
    let mut _windows : Windows = Windows::new(asmcpu, virpc);

    let shared_ch = Arc::new(AtomicIsize::new(0));
    let key_handle = keyboard_thread(shared_ch.clone());

    let mut ch = 0;
    while ch != 27 as i32 { // ESC pressed, so quit
        //load new char
        ch = shared_ch.load(Ordering::Relaxed) as i32;
        //reset ch, so that nex one can be loaded
        shared_ch.store(0,Ordering::Relaxed);

        //run emulator
        _windows.run_virpc();

        //run IDE
        _windows.refresh_fast();
        _windows.resize_check();
        //handle keys
        if ch > 0 {
            _windows.handle_keys(ch);
        }
        //wait a bit
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    key_handle.join().unwrap();
    _windows.destroy();
}

fn keyboard_thread(ch : Arc<AtomicIsize>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut l_ch = 0;
        while l_ch != 27 { // ESC pressed, so quit, TODO: make this a semaphore
            //retrieve a new character (blocking)
            l_ch = getch();

            //wait until previous character was processed
            while ch.load(Ordering::Relaxed) > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            //pass next character to main thread
            ch.store(l_ch as isize, Ordering::Relaxed);

            //wait a bit
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    })
}