//! # Element
//! 基本绘图元素
//! 2020年12月30日 zg

use crate::{memory::block::Block, virtio::gpu_device::{Pixel, Rect}};

/// ## Element
/// 基本显示元素，x、y 为左上角顶点
pub struct Element{
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
    pub content : Block,
}

pub trait Draw {
    fn draw(&self);
    fn draw_area(&self, rect : Rect);
    fn fill_font(&self, c : usize, x : usize, y : usize, foreground : Pixel, background : Pixel);
}




