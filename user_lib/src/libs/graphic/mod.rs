//! # 图形绘制底层
//! 封装基本图形操作
//! 
//! 2021年1月31日 zg

#![allow(dead_code)]


mod colorblock;
mod canvas;
mod font;
mod config;

pub use colorblock::*;
pub use canvas::*;
pub use font::*;
pub use config::*;

use tisu_driver::Pixel;

use crate::libs::syscall::draw_rect;


pub fn draw_rect_override(rect:Rect, buffer : &[Pixel]) {
    draw_rect(
    rect.x1 as usize,rect.y1 as usize,rect.x2 as usize,rect.y2 as usize,buffer);
}

pub fn draw_rect_blend(rect:Rect, buffer : &[Pixel]) {
    draw_rect(
    rect.x1 as usize,rect.y1 as usize,rect.x2 as usize,rect.y2 as usize,buffer);
}

pub fn invalid() {}