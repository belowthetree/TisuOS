//! # 变换用结构工具
//! 
//! 2021年1月25日 zg

#![allow(dead_code)]

#[derive(Clone, Copy, Debug)]
pub struct Vector {
    pub x : isize,
    pub y : isize,
}

impl Vector {
    pub fn new(x : isize, y : isize)->Self {
        Self {
            x : x,
            y : y,
        }
    }
    pub fn from(point : ScalePoint)->Self {
        Self {
            x : (point.x * WIDTH as f32) as isize,
            y : (point.y * HEIGHT as f32) as isize,
        }
    }

}


#[derive(Clone, Copy, Debug)]
pub struct Position {
    pub x : usize,
    pub y : usize,
}

impl Position {
    pub fn new(x : usize, y : usize)->Self {
        Self {
            x : x,
            y : y,
        }
    }
    pub fn new_default()->Self {
        Self {
            x : 0,
            y : 0,
        }
    }
    pub fn from_scale_point(p : ScalePoint)->Self {
        Self {
            x : (p.x * WIDTH as f32) as usize,
            y : (p.y * HEIGHT as f32) as usize,
        }
    }
}

pub struct OffsetPosition {
    pub x : isize,
    pub y : isize,
}

#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x1 : u32,
    pub y1 : u32,
    pub x2 : u32,
    pub y2 : u32,
}

pub struct Area {
    pub x1 : usize,
    pub y1 : usize,
    pub x2 : usize,
    pub y2 : usize,
}

impl Area {
    pub fn new(x : usize, y : usize, width : usize, height : usize)->Self {
        Self {
            x1 : x,
            y1 : y,
            x2 : x + width,
            y2 : y + height,
        }
    }
    pub fn inside(&self, point : Position)->bool {
        self.x1 <= point.x && self.y1 <= point.y && self.x2 >= point.x && self.y2 >= point.y
    }
}

/// ## 比例坐标点
/// 0 ~ 1，表示在屏幕中的位置比例
#[derive(Clone, Copy, Debug)]
pub struct ScalePoint {
    pub x : f32,
    pub y : f32,
}
impl ScalePoint {
    pub const fn new()->Self{
        Self{
            x : 0.0,
            y : 0.0,
        }
    }
}

impl PartialEq for ScalePoint{
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == self.y
    }
}

impl Sub for ScalePoint{
    type Output = Self;
    fn sub(self, rhs: Self)->Self {
        Self{
            x : self.x - rhs.x,
            y : self.y - rhs.y,
        }
    }
}


use core::{ops::Sub, usize};

use crate::virtio::gpu_device::{HEIGHT, WIDTH};

