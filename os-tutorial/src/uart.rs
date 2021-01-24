//! # uart.rs
//! UART 可以直接传输用户的输入输出
//! 作为前期输出调试的主要手段，必须优先实现
//! 
//! 2020年12月 zg

use core::fmt::{Write, Error};
use core::convert::TryInto;

pub const UART_ADDR : usize = 0x1000_0000;
const LSR_OFFSET : usize = 5;

pub struct Uart;
/// 继承 Write Trait 使得 print 宏得以使用
/// 字符转换等由 Rust 提供，非常方便
impl Write for Uart {
	fn write_str(&mut self, out: &str) -> Result<(), Error> {
		for c in out.bytes() {
			self.put(c);
		}
		Ok(())
	}
}

impl Uart {
    pub fn new() -> Self {
        Uart
    }
    /// ## 初始化 UART
    /// 主要包括：
    /// lcr：每次传输的数据位数
    /// fcr：先进先出
    /// 激活
    pub fn init(&mut self) {
        unsafe {
            let ptr = UART_ADDR as *mut u8;
            // 偏移 3 指出每次传输的位数，恒定 8 位即一字节
            ptr.add(3).write_volatile(8);
            // 激活 FIFI
            ptr.add(2).write_volatile(1);
            // 激活中断
            ptr.add(1).write_volatile(1);
            // 设置输入产生的中断频率
            let divisor : u16 = 592;
			let divisor_least: u8 = (divisor & 0xff).try_into().unwrap();
			let divisor_most:  u8 = (divisor >> 8).try_into().unwrap();
            let lcr = ptr.add(3).read_volatile();
            ptr.add(3).write_volatile(lcr | 1 << 7);
            
            ptr.add(0).write_volatile(divisor_least);
            ptr.add(1).write_volatile(divisor_most);
            ptr.add(3).write_volatile(lcr);
        }
    }
    /// ## 获取键盘输入
    /// 从 MMIO 对应地址获取输入
    pub fn get(&self) -> Option<u8> {
        unsafe {
            let ptr = UART_ADDR as *mut u8;
            if ptr.add(LSR_OFFSET).read_volatile() & 1 == 0 {
                None
            }
            else {
                Some(ptr.read_volatile())
            }
        }
    }
    /// ## 输出
    /// 通过 MMIO 的方式
    pub fn put(&mut self, c : u8) {
        unsafe {
            let ptr = UART_ADDR as *mut u8;
            ptr.add(0).write_volatile(c);
        }
    }
}

