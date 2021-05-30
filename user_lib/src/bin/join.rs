#![no_std]
#![no_main]

use user_lib::libs::syscall::{fork, get_tid, join, set_timer, wait};

#[macro_use]
extern crate user_lib;

#[no_mangle]
extern "C" fn _start(){
    if fork() == 0 {
        println!("tid {} is sleep", get_tid());
        set_timer(1000_0000);
    }
    else {
        if fork() == 0 {
            println!("tid {} is sleep", get_tid());
            set_timer(2000_0000);
        }
        else {
            if fork() == 0 {
                println!("tid {} is sleep", get_tid());
                set_timer(3000_0000);
            }
            else {
                if fork() == 0 {
                    println!("tid {} is sleep", get_tid());
                    set_timer(4000_0000);
                }
            }
        }
    }
    println!("tid {} finish", get_tid());
    join();
}