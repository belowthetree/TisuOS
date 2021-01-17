//! # 桌面接口
//! 要求桌面元素全体支持，接口与底层 Graphic 紧密结合
//! 2020年12月30日 zg

use crate::virtio::input::input_buffer::Point;

pub trait Transform {
    fn set_position(&mut self, x : u32, y : u32);
    fn translate(&mut self, x : i32, y : i32);
    fn maximum(&mut self);
    fn minimum(&mut self);
    fn detect(&self, point : Point)->bool;
}

