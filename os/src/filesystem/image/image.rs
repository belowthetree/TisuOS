//! 图片
//! 所有图片格式最终转换成此接口
//! 
//! 2020年12月31日 zg


pub struct Image{
    pub width : usize,
    pub height : usize,
    pub format : Format,
    pub data : Block,
}

/// 默认以 Pixel 为单位读写
impl Image {
    pub fn new(width : usize, height : usize, format : Format)->Self{
        let t = Block::new(width * height * size_of::<Pixel>());
        Self{
            width : width,
            height : height,
            format : format,
            data : t,
        }
    }
    #[allow(dead_code)]
    pub fn get(&self, idx : usize)->Pixel{
        let ptr = self.data.addr as *mut Pixel;
        unsafe {
            *ptr.add(idx)
        }
    }
    pub fn set(&mut self, idx : usize, color : Pixel){
        unsafe {
            let ptr = self.data.addr as *mut Pixel;
            ptr.add(idx).write_volatile(color);
        }
    }
    /// 上下颠倒
    pub fn updown(&mut self){
        let count = self.width * self.height * size_of::<Pixel>();
        let b = Block::new(count);
        unsafe {
            b.addr.copy_from(self.data.addr, count);
            for y in 0..self.height {
                self.data.addr.add(y * self.width * size_of::<Pixel>()).
                copy_from(b.addr.add(count - (y + 1) * self.width * size_of::<Pixel>()), self.width * size_of::<Pixel>());
            }
        }
    }
}

pub enum Format{
    RGB,
    // RGBA,
}


use core::mem::size_of;
use crate::{memory::block::{Block}, virtio::gpu_device::Pixel};

