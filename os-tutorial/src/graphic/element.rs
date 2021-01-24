//! # Element
//! 基本绘图元素
//! 2020年12月30日 zg

use crate::{filesystem::image::image::Image, memory::block::Block, virtio::gpu_device::{Pixel, Rect}};

/// ## Element
/// 基本显示元素，x、y 为左上角顶点
pub struct Element{
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
    pub content : Block,
}

pub trait Draw {
    /// 直接覆盖
    fn draw(&self);
    /// 带透明度
    fn draw_blend(&self);
    fn draw_offset(&self, rect : Rect);
    fn fill_font(&mut self, c : usize, x : usize, y : usize, foreground : Pixel, background : Pixel);
    fn fill_image(&mut self, rect : Rect, image : Image);
}




