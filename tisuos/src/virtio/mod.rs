//! # 虚拟 IO 设备
//! 包括块设备、键盘、鼠标、GPU 等设备的控制
//!
//! 2020年11月 zg


pub mod device;
pub mod input_buffer;
pub mod config;
pub mod disk_cache;
pub mod ip;

pub fn init() {
    device::init();
}