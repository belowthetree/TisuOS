//! # 图形结构
//! 
//! 2021年1月25日 zg

#![allow(dead_code)]
pub use tisu_driver::Pixel;


/// ## 颜色字符
#[derive(Clone, Copy)]
pub struct ColorChar{
    pub color : Pixel,
    pub c : char,
}

