#![no_std]
#![feature(
    asm,
    panic_info_message,
    lang_items,
    allocator_api,
    alloc_error_handler,
    alloc_prelude,
    untagged_unions,
    core_intrinsics,
)]

#[macro_use]
extern crate alloc;
use alloc::{prelude::v1::*};
use interact::shell;
use core::{panic::PanicInfo};

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!(uart::Uart::new(), $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
			print!(concat!($fmt, "\r\n"))
			});
	($fmt:expr, $($args:tt)+) => ({
			print!(concat!($fmt, "\r\n"), $($args)+)
			});
}

#[lang = "eh_personality"] extern fn eh_personality() {}

#[panic_handler]
fn panic(_info :& PanicInfo) -> ! {
    println!("Panic! ");
    if let Some(p) = _info.location() {
		println!(
				"line {}, file {}: {}",
				p.line(),
				p.file(),
				_info.message().unwrap()
				);
	}
    else {
        println!("no information");
    }
    abort();
}

#[no_mangle]
extern "C" fn abort() -> !{
    loop{
        unsafe{
            asm!("wfi"::::"volatile");
        }
    }
}

#[no_mangle]
extern "C" fn kernel_init(){
    Uart::new().init();
    trap::init(0);
    page::init();
    global_allocator::init();
    println!("trap stack end {:x},
        kernel heap {:x}, kernel stack {:x}",
        unsafe{page::TRAP_STACK_END}, unsafe {page::KERNEL_HEAP_START}, unsafe {page::KERNEL_STACK_END});
    println!("finish mem");
    plic::init();
    println!("before device");
    device::init();
    println!("finish device");
    
    process::init();
	buffer::init();
    // test_disk();
    // abort();
    operation::init(); // 文件系统初始化
    global_allocator::Memory::print();
    println!("finish filesystem");
    input::init();

    gpu_device::reset(0);
    println!("finish gpu");

    shell::init();
    input_buffer::init();
    desktop::desktop::init();
    timer::set_next_interrupt(1);
    process::start_init_process();
    // process::create_and_add(shell::update as usize, true);


    println!("to process");
}

#[no_mangle]
extern "C" fn kernel_start(hartid : usize){
    trap::init(hartid);
}
#[allow(dead_code)]
fn test_disk(){
    println!("start test");
    let buffer1 = global_allocator::alloc(2048, true);
    for i in 0..10{
        unsafe {
            buffer1.add(i).write_volatile(i as u8 + 2);
        }
    }
    sync_write(0, buffer1, 512, 0);
    buffer::sync_write_buffer(0, buffer1, 512, 0);
    buffer::sync_write_buffer(0, buffer1, 512, 1024 * 4096);
    for i in 0..10{
        unsafe {
            print!("{:x} ", buffer1.add(i).read_volatile());
        }
    }
    buffer::sync_read_buffer(0, buffer1, 512, 0);
    for i in 0..10{
        unsafe {
            print!("{:x} ", buffer1.add(i).read_volatile());
        }
    }

    global_allocator::free(buffer1);
    println!("finish disk test");
}
#[allow(dead_code)]
fn test_global_allocator(){
    println!("test global_allocator alloc");
    let a1 = global_allocator::alloc(4096, true);
    page::print_page_alloc();
    global_allocator::Memory::print();
    let a2 = global_allocator::alloc(4096, true);
    let a3 = global_allocator::alloc(4096, true);
    let a4 = global_allocator::alloc(4096, true);

    let b1 = global_allocator::alloc(4096, false);
    let b2 = global_allocator::alloc(4096, false);
    let b3 = global_allocator::alloc(4096, false);
    let b4 = global_allocator::alloc(4096, false);

    println!("before delete");
    page::print_page_alloc();
    global_allocator::Memory::print();

    global_allocator::free(a1);
    global_allocator::free(a2);

    global_allocator::free(b1);
    global_allocator::free(b2);

    println!("after delete");
    page::print_page_alloc();
    global_allocator::Memory::print();
    global_allocator::free(b3);
    global_allocator::free(b4);
    global_allocator::free(a3);
    global_allocator::free(a4);

    println!("test kernel container");

    {
        println!("before container");
        page::print_page_alloc();
        global_allocator::Memory::print();
        let b = Box::new(10);
        println!("alloc box {}", *b);
        let v = vec![1, 3, 5];
        println!("vec values: ");
        for i in v{
            print!("{} ", i);
        }
        println!();
        println!("after container");
        page::print_page_alloc();
        global_allocator::Memory::print();
    }

    let mut v = Vec::<i32>::new();
    for i in 0..10 {
        v.push(i);
    }
    for i in v{
        print!("{}", i);
    }
    println!();
}
// fn test_syscall(){
//     let v = Vec::<u32>::new();
//     for _ in 0..
// }
mod uart;
mod cpu;
mod interrupt;
mod memory;
mod task;
mod plic;
mod virtio;
mod sync;
mod filesystem;
mod interact;
mod graphic;
mod desktop;
mod libs;
use task::{process};
use interact::input;
use filesystem::{operation};
use virtio::{block_device::sync_write, buffer, device, gpu_device, input::input_buffer};
use memory::global_allocator;
use memory::page;
use interrupt::trap;
use interrupt::timer;
use uart::Uart;
