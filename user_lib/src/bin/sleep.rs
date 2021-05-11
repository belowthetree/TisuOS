#![no_std]
#![no_main]

use user_lib::libs::syscall::{set_timer, syscall_test};

extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    syscall_test();
    set_timer(2000_0000);
    syscall_test();
}