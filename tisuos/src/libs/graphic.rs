//! # 图形结构
//! 
//! 2021年1月25日 zg

#![allow(dead_code)]

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct Pixel {
	pub r: u8,
	pub g: u8,
	pub b: u8,
	pub a: u8,
}

impl Pixel{
    pub fn new(r:u8, g:u8, b:u8, a:u8)->Self {
        Self {
            r : r,
            g : g,
            b : b,
            a : a,
        }
    }
    pub const fn red()->Self{
        Pixel{
            r:255,
            g:0,
            b:0,
            a:255,
        }
    }
    pub const fn green()->Self{
        Self{
            r:0,
            g:255,
            b:0,
            a:255
        }
    }
    pub const fn blue()->Self{
        Self{
            r:0,
            g:0,
            b:255,
            a:255
        }
    }
    pub const fn yellow()->Self{
        Self{
            r:255,
            g:255,
            b:0,
            a:255
        }
    }
    pub const fn grey()->Self{
        Self{
            r:55,
            g:55,
            b:55,
            a:255,
        }
    }
    pub const fn white()->Self{
        Self{
            r:255,
            g:255,
            b:255,
            a:255
        }
    }
    pub const fn black()->Self{
        Self{
            r : 0,
            g : 0,
            b : 0,
            a : 255,
        }
    }
    pub const fn shallow_grey()->Self{
        Self{
            r:122,
            g:122,
            b:122,
            a:255,
        }
    }
}
