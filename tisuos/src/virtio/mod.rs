//! # 虚拟 IO 设备
//! 包括块设备、键盘、鼠标、GPU 等设备的控制
//! 2020年11月 zg

pub mod device;
pub mod block_device;
pub mod gpu_device;
pub mod input;
pub mod buffer;
use input::input_device;

pub fn init() {
    device::init();
    buffer::init();
}