//! # Input
//! 处理命令行的输入，直接送给标准输入
//!
//! 2020年12月20日 zg

use core::fmt::Write;

/// ## 驱动全局输入
/// 由 PLIC 驱动
pub fn pending(){
    push_input(Uart::new().get().unwrap() as char);
}

pub struct Console{}

impl Console {
    pub fn new()->Self{Self{}}
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            push_output(c as char);
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! console {
    () => ({});
    ($($args:tt)+) => ({
        use core::fmt::Write;
        let _ = write!($crate::interact::console_input::Console::new(), $($args)+);
    });
}

pub fn output_handler() {
    // let uart = Uart::new();
    unsafe {
        loop {
            asm!("wfi");
            while let Some(c) = pop_output() {
                print!("{}", c);
            }
        }
    }
}

// use super::shell;
use crate::{filesystem::{pop_output, push_input, push_output}, uart::Uart};