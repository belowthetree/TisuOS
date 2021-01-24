//! # Button
//! 按钮控件
//! 2020年12月30日 zg

#![allow(dead_code)]
/// ## 按钮应该维护一个图标来源，可以是图也可以是代码生成的颜色样式
/// 虽然也具有坐标、深度等属性，但是与 Graphic 元素的属性是不同的意义
pub struct Button{
    pub background : Style,
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
    pub click : bool,
}

pub const BUTTON_WIDTH : usize = 30;

impl Button{
    pub fn new(x : usize, y : usize, width : usize, height : usize)->Self{
        let mut background = Style::new(ColorStyle::SolidColor, x, y, width, height);
        background.set_color(Pixel::red());
        Self{
            background : background,
            x : x,
            y : y,
            width : width,
            height : height,
            click : false,
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
            click : false,
        }
    }
    pub fn resize(&mut self, width : usize, height : usize) {
        self.background.resize(width, height);
    }
    pub fn set_color(&mut self, color :Pixel){
        self.background.set_color(color);
    }
    pub fn draw(&self){
        self.background.draw();
    }
    pub fn draw_blend(&self){
        self.background.draw_blend();
    }
    pub fn set_texture(&mut self, image : Image){
        self.background.set_texture(image);
    }
}

impl Transform for Button{
    fn set_position(&mut self, x : usize, y : usize) {
        self.x = x;
        self.y = y;
        self.background.element.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&mut self, point : Position)->bool {
        let x = point.x;
        let y = point.y;
        let rt = self.x <= x && self.y <= y && self.x + self.width >= x && self.y + self.height >= y;
        if rt {
            self.click = true;
        }
        rt
    }

    fn translate(&mut self, x : isize, y : isize) {
        let mut xx = self.x as isize + x;
        let mut yy = self.y as isize + y;
        if xx < 0{
            xx = 0;
        }
        if yy < 0{
            yy = 0;
        }
        self.x = xx as usize;
        self.y = yy as usize;
        self.background.translate(x, y);
    }

    fn refresh(&mut self) {
    }
}




use desktop_trait::Transform;

use crate::{desktop::desktop::Position, filesystem::image::image::Image};
use crate::{desktop::desktop_trait, virtio::gpu_device::{Pixel}};
use crate::graphic::transform::ElemTranform;
use super::style::{style::{ColorStyle, Style}};


