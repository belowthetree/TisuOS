//! # Button
//! 按钮控件
//! 2020年12月30日 zg

#![allow(dead_code)]
/// ## 按钮应该维护一个图标来源，可以是图也可以是代码生成的颜色样式
/// 虽然也具有坐标、深度等属性，但是与 Graphic 元素的属性是不同的意义
pub struct Button{
    pub background : Style,
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
}

pub const BUTTON_WIDTH : u32 = 30;

impl Button{
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        let mut background = Style::new(ColorStyle::SolidColor, x, y, width, height);
        background.set_color(Pixel::red());
        Self{
            background : background,
            x : x,
            y : y,
            width : width,
            height : height,
        }
    }
    pub fn new_default()->Self{
        let mut back = Style::new_default();
        back.set_color(Pixel::red());
        Self{
            background : back,
            x : 0,
            y : 0,
            width : BUTTON_WIDTH,
            height : 20,
        }
    }
    pub fn resize(&mut self, width : u32, height : u32) {
        self.background.resize(width, height);
    }
    pub fn set_color(&mut self, color :Pixel){
        self.background.set_color(color);
    }
    pub fn draw(&self){
        self.background.draw_area();
    }
}

impl Transform for Button{
    fn set_position(&mut self, x : u32, y : u32) {
        self.x = x;
        self.y = y;
        self.background.element.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&self, point : Point)->bool {
        false
    }

    fn translate(&mut self, x : i32, y : i32) {
        self.background.translate(x, y);
    }
}




use core::cmp::{max, min};
use desktop_trait::Transform;

use crate::{uart, virtio::input::input_buffer::Point};
use crate::{desktop::desktop_trait, virtio::gpu_device::{HEIGHT, Pixel, WIDTH}};
use crate::graphic::transform::ElemTranform;
use super::style::style::{ColorStyle, Style};


