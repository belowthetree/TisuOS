#![no_std]
#![feature(
    asm,
    global_asm,
    panic_info_message,
    lang_items,
    linkage,
    alloc_error_handler,
    alloc_prelude,
)]
#![allow(dead_code)]

extern crate alloc;

mod lang_items;
pub mod libs;
mod memory;
#[macro_use]
pub mod stdio;

pub use stdio::Stdio;
use tisu_sync::SpinMutex;

#[linkage = "weak"]
#[no_mangle]
fn main(_argc: usize) -> i32 {
    panic!("Cannot find main!");
}

pub static mut MUTEX : SpinMutex = SpinMutex::new();

#[macro_export]
macro_rules! print {
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!($crate::Stdio::new(), $($args)+);
    });
}

#[macro_export]
macro_rules! println
{
	() => ({
		   print!("\r\n")
		   });
	($fmt:expr) => ({
        unsafe {
            $crate::MUTEX.lock();
			$crate::print!(concat!($fmt, "\r\n"));
            $crate::MUTEX.unlock();
        }
			});
	($fmt:expr, $($args:tt)+) => ({
        unsafe {
            $crate::MUTEX.lock();
			$crate::print!(concat!($fmt, "\r\n"), $($args)+);
            $crate::MUTEX.unlock();
        }
        });
}