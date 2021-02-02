//! 图片
//! 所有图片格式最终转换成此接口
//! 
//! 2020年12月31日 zg


pub struct Image{
    pub width : usize,
    pub height : usize,
    pub format : Format,
    pub data : Block<Pixel>,
}

/// ## 默认以 Pixel 为单位读写
impl Image {
    pub fn new(width : usize, height : usize, format : Format)->Self{
        let t = Block::<Pixel>::new(width * height);
        Self{
            width : width,
            height : height,
            format : format,
            data : t,
        }
    }
    #[allow(dead_code)]
    pub fn get(&self, idx : usize)->Pixel{
        self.data.get(idx).unwrap()
    }
    pub fn set(&mut self, idx : usize, color : Pixel){
        self.data.set(idx, color, 1);
    }
    /// ### 上下颠倒
    pub fn updown(&mut self){
        let count = self.width * self.height * size_of::<Pixel>();
        let b = Block::<Pixel>::new(count);
        b.copy_from(0, &self.data, 0, count);
        for y in 0..self.height {
            self.data.copy_from(y * self.width, 
            &b, count - (y + 1) * self.width, self.width);
        }
    }
    pub fn resize(&mut self, width : usize, height : usize) {
        let tm = Block::<Pixel>::new(width * height);
        for y in 0..height {
            let yy = y * self.height / height;
            let t1 = y * width;
            let t2 = yy * self.width;
            for x in 0..width {
                let xx = x * self.width / width;
                let v = self.data.get(xx + t2).unwrap();
                tm.set(x + t1, v, 1);
            }
        }
        self.width = width;
        self.height = height;
        self.data = tm;
    }
}

pub enum Format{
    RGB,
    // RGBA,
}


use core::mem::size_of;
use crate::{libs::graphic::Pixel, memory::block::{Block}};

