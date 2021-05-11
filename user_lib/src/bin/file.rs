#![no_std]
#![no_main]
#![feature(
    alloc_prelude,
    asm,
)]
extern crate alloc;
#[macro_use]
extern crate user_lib;
extern crate fs_format;

use core::ptr::slice_from_raw_parts_mut;

use alloc::prelude::v1::*;
use fs_format::{ELF, FileInfo};
use user_lib::libs::syscall::{file_info, free, malloc, open, read, set_timer};

#[no_mangle]
extern "C" fn _start(){
    let id = open("0/alloc.elf".to_string(), 1);
    println!("get file id {}", id);
    if id > 0 {
        let addr = file_info("0/alloc.elf".to_string());
        let info = addr as *const FileInfo;
        let info = unsafe{(*info).clone()};
        println!("{:?}", info);
        let ptr = malloc(info.size) as *mut u8;
        let data = unsafe {&mut *(slice_from_raw_parts_mut(ptr, info.size))};
        let len = read(info.id, data);
        println!("read len {}", len);
        if len > 0 {
            let elf = unsafe {&*(ptr as *const ELF)};
            println!("{:?}", elf);
        }
        free(ptr as usize);
    }
    set_timer(0x100_0000);
}
