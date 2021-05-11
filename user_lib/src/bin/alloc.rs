#![no_std]
#![no_main]
#![feature(
    alloc_prelude,
)]
extern crate alloc;
#[macro_use]
extern crate user_lib;

use alloc::prelude::v1::*;

#[no_mangle]
extern "C" fn _start(){
    let mut v = Vec::new();
    for i in 0..10 {
        v.push(i);
    }
    println!("before for");
    for i in v {
        print!("{} ", i);
    }
    println!();
}