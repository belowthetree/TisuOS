#![no_std]
#![no_main]

use user_lib::syscall_test;


extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    syscall_test();
}