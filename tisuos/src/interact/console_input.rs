//! # Input
//! 处理命令行的输入，直接送给标准输入
//!
//! 2020年12月20日 zg

/// ## 驱动全局输入
/// 由 PLIC 驱动
pub fn pending(){
    push_input(Uart::new().get().unwrap() as char);
}

pub fn output_handler() {
    let uart = Uart::new();
    unsafe {
        loop {
            asm!("wfi");
            while let Some(c) = pop_output() {
                uart.put(c as u8)
            }
        }
    }
}

// use super::shell;
use crate::{filesystem::{pop_output, push_input}, uart::Uart};