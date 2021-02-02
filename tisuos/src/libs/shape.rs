//! # 变换用结构工具
//! 
//! 2021年1月25日 zg

#![allow(dead_code)]



pub struct Position {
    pub x : usize,
    pub y : usize,
}

pub struct OffsetPosition {
    pub x : isize,
    pub y : isize,
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x1 : u32,
    pub y1 : u32,
    pub x2 : u32,
    pub y2 : u32,
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
    fn sub(self, rhs: Self) ->Self {
        Self{
            x : rhs.x - self.x,
            y : rhs.y - self.y,
        }
    }
}


use core::ops::Sub;

