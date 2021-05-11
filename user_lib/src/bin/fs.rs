#![no_std]
#![no_main]
#![feature(
    alloc_prelude,
)]

use user_lib::libs::fs::Directory;

#[macro_use]
extern crate user_lib;
extern crate alloc;
use alloc::prelude::v1::*;

#[no_mangle]
extern "C" fn _start(){
    let dir = Directory::read("0/".to_string());
    for file in dir.file.iter() {
        print!("{} ", file);
    }
    println!()
}