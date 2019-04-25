// helper utility functions and macros

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::io::SeekFrom;
use std::path::Path;


// helper macros to easily extract references from Option<RefCell<...>>
macro_rules! as_ref {
    ($x:expr) => ($x.as_ref().unwrap().borrow_mut())
}

macro_rules! as_mut {
    ($x:expr) => ($x.as_mut().unwrap().borrow_mut())
}

// common helper functions
pub fn open_file(filename: &str, offset: u64) -> Vec<u8> {
    let path = Path::new(&filename);
    
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", path.display(), Error::description(&why)),
        Ok(file) => file,
    };

    let mut file_data = Vec::<u8>::new();

    let _ = file.seek(SeekFrom::Start(offset));
    let result = file.read_to_end(&mut file_data);
    
    match result {
        Err(why)   => panic!("Error reading file: {}", Error::description(&why)),
        Ok(result) => println!("Read {}: {} bytes", path.display(), result),
    };    

    file_data
}


// set 8 consecutive buffer elements to single value for faster update of
// a single 8-pixel screen chunk
pub fn memset8(buffer: &mut [u32], start: usize, value: u32) {
    buffer[start]   = value;
    buffer[start+1] = buffer[start];
    buffer[start+2] = buffer[start];
    buffer[start+3] = buffer[start];
    buffer[start+4] = buffer[start];
    buffer[start+5] = buffer[start];
    buffer[start+6] = buffer[start];
    buffer[start+7] = buffer[start];
}

pub fn fetch_c64_color_rgba(idx: u8) -> u32 {
    // palette RGB values copied from WinVICE
    match idx & 0x0F {
        0x00  => 0x00000000,
        0x01  => 0x00FFFFFF,
        0x02  => 0x00894036,
        0x03  => 0x007ABFC7,
        0x04  => 0x008A46AE,
        0x05  => 0x0068A941,
        0x06  => 0x003E31A2,
        0x07  => 0x00D0DC71,
        0x08  => 0x00905F25,
        0x09  => 0x005C4700,
        0x0A  => 0x00BB776D,
        0x0B  => 0x00555555,
        0x0C  => 0x00808080,
        0x0D  => 0x00ACEA88,
        0x0E  => 0x007C70DA,
        0x0F  => 0x00ABABAB,
        _ => panic!("Unknown color!"),
    }
}

