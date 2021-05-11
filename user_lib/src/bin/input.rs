#![no_std]
#![no_main]

use user_lib::libs::{stdio, syscall::set_timer};

#[macro_use]
extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    loop {
        if let Some(s) = stdio::get_input() {
            println!("{}", s);
            break;
        }
    }
    set_timer(1000_0000)
}