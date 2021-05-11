#![no_std]
#![no_main]
#![feature(
    alloc_prelude,
)]

use user_lib::libs::syscall::set_timer;
use user_lib::println;

extern crate alloc;
extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    for i in 0..10 {
        println!("stdout {}", i);
        set_timer(1000_0000);
    }
}