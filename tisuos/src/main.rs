#![no_std]
#![no_main]
#![feature(
    asm,
    panic_info_message,
    lang_items,
    allocator_api,
    alloc_error_handler,
    alloc_prelude,
    untagged_unions,
    core_intrinsics,
    const_fn,
    global_asm,
    map_first_last,
)]

global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/mem.S"));
global_asm!(include_str!("asm/trap.S"));
global_asm!(include_str!("asm/strap.S"));

#[macro_use]
extern crate alloc;

pub static mut M : SpinMutex = SpinMutex::new();

#[no_mangle]
extern "C" fn kernel_init(){
    Uart::new().init();
    trap::init(0);
    strap::init(0);
    memory::init();
    // memory::test();
    plic::init();
    task::init();
    input_buffer::init();
    virtio::init();

    process::start_init_process();
}

#[no_mangle]
extern "C" fn kernel_start(hartid : usize){
    trap::init(hartid);
    strap::init(hartid);
}

#[macro_use]
mod uart;
mod plic;
mod cpu;
mod memory;
mod task;
mod libs;
mod interrupt;
mod interact;
mod virtio;
mod filesystem;
mod graphic;
mod panic;
mod desktop;
mod rtc;
use interrupt::{strap, trap};
use task::process;
use tisu_sync::SpinMutex;
use uart::Uart;
use virtio::input_buffer;
