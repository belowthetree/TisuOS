#![no_std]
#![no_main]

extern crate user_lib;
use user_lib::libs::syscall::{branch, syscall_test, syscall_test2, wait};

#[no_mangle]
extern "C" fn _start(){
    let id = branch(test as usize);
    wait(id);
    syscall_test2();
}

fn test() {
    syscall_test()
}