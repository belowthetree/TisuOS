#![no_std]
#![feature(
    asm,
    panic_info_message,
    alloc_prelude,
    alloc_error_handler,
)]

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
    trap::init();
    page::init();
    global_allocator::init();
    test_global_allocator();
    process::init();
    timer::set_next_interrupt(1);
    process::start_init_process();
}

fn test_global_allocator(){
    println!("test global_allocator alloc");
    let a1 = global_allocator::alloc_kernel(4096);
    page::print_page_alloc();
    global_allocator::Memory::print();
    let a2 = global_allocator::alloc_kernel(4096);
    let a3 = global_allocator::alloc_kernel(4096);
    let a4 = global_allocator::alloc_kernel(4096);

    let b1 = global_allocator::alloc_user(4096);
    let b2 = global_allocator::alloc_user(4096);
    let b3 = global_allocator::alloc_user(4096);
    let b4 = global_allocator::alloc_user(4096);

    println!("before delete");
    page::print_page_alloc();
    global_allocator::Memory::print();

    global_allocator::free_kernel(a1);
    global_allocator::free_kernel(a2);

    global_allocator::free_user(b1);
    global_allocator::free_user(b2);

    println!("after delete");
    page::print_page_alloc();
    global_allocator::Memory::print();
    global_allocator::free_user(b3);
    global_allocator::free_user(b4);
    global_allocator::free_kernel(a3);
    global_allocator::free_kernel(a4);

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

#[no_mangle]
extern "C" fn kernel_start(){
}

#[macro_use]
extern crate alloc;
use alloc::{prelude::v1::*};

use core::panic::PanicInfo;
mod uart;
mod interrupt;
mod cpu;
mod memory;
mod task;
mod sync;
use task::process;
use memory::global_allocator;
use memory::page;
use uart::Uart;
use interrupt::{trap, timer};