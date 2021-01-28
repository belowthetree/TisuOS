//! # 变换用结构工具
//! 
//! 2021年1月25日 zg

#![allow(d)]

pub struct ScalePoint {
    pub x : f32,
    pub y : f32,
}

pub struct Position {
    pub x : usize,
    pub y : usize,
}

pub struct Rect {
    pub x1 : usize,
    pub y1 : usize,
    pub x2 : usize,
    pub y2 : usize,
}
