//! # 按钮
//! 
//! 2021年2月2日 zg


pub struct Button {
    pub area : Area,
    pub background : ColorBlock,
}

impl Button {
    pub fn new(x : usize, y : usize, width : usize, height : usize, color : Pixel)->Self {
        Self {
            area : Area::new(x, y, width, height),
            background : ColorBlock::solid_color(x, y, width, height, color),
        }
    }
    pub fn image(x : usize, y : usize, width : usize, height : usize, image : &Image)->Self {
        Self {
            area : Area::new(x, y, width, height),
            background : ColorBlock::image(x, y, image),
        }
    }
    pub fn draw(&self) {
        self.background.draw_override();
    }
    pub fn draw_blend(&self) {
        self.background.draw_blend();
    }
}

impl Button {
    pub fn translate(&mut self, vec : Vector) {
        let x = max(0, self.area.x1 as isize + vec.x) as usize;
        let y = max(0, self.area.y1 as isize + vec.y) as usize;
        self.area.x2 = x + self.area.x2 - self.area.x1;
        self.area.y2 = y + self.area.y2 - self.area.y1;
        self.area.x1 = x;
        self.area.y1 = y;
        self.background.x = x;
        self.background.y = y;
    }
    pub fn detect(&self, point : Position)->bool {
        self.area.inside(point)
    }
    pub fn set_position(&mut self, pos : Position) {
        let dx = pos.x as isize - self.area.x1 as isize;
        let dy = pos.y as isize - self.area.y1 as isize;
        self.area.x1 = pos.x;
        self.area.y1 = pos.y;
        self.area.x2 = (self.area.x2 as isize + dx) as usize;
        self.area.y2 = (self.area.y2 as isize + dy) as usize;
        self.background.x = pos.x;
        self.background.y = pos.y;
    }
    pub fn resize(&mut self, width : usize, height : usize) {
        self.background.resize(width, height);
    }
}


use core::cmp::max;

use crate::{filesystem::image::image::Image, graphic::colorblock::ColorBlock, libs::{graphic::Pixel, shape::{Area, Position, Vector}}};


