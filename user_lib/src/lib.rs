#![no_std]
#![feature(
    asm,
    global_asm,
    panic_info_message,
    lang_items,
    linkage,
)]

mod lang_items;
mod libs;

pub use libs::syscall::*;

#[no_mangle]
extern "C" fn start(){
    main(0);
}

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize) -> i32 {
    syscall_test();
    panic!("Cannot find main!");
}
