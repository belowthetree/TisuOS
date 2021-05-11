#![no_std]
#![no_main]

use user_lib::libs::syscall::{fork, syscall_test, syscall_test2, wait};

extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    let t = fork();
    if t == 0 {
        syscall_test();
    }
    else {
        wait(t);
        syscall_test2()
    }
}