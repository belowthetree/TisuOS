//! # Element
//! 基本绘图元素
//! 2020年12月30日 zg

use crate::memory::block::Block;

/// ## Element
/// 基本显示元素，x、y 为左上角顶点
pub struct Element{
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
    pub content : Box<Block>,
}

pub trait Draw {
    fn draw_area(&self);
}




use alloc::{prelude::v1::*};

