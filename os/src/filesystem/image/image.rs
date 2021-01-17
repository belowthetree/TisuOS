//! Image
//! 所有图片格式最终转换成此接口
//! 2020年12月31日 zg


pub struct Image{
    pub width : usize,
    pub height : usize,
    pub format : Format,
    pub data : Box<Block>
}

/// 默认以 Pixel 为单位读写
impl Image {
    pub fn new(width : usize, height : usize, format : Format)->Self{
        let t = new_block(width * height * size_of::<Pixel>());
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
}

pub enum Format{
    RGB,
    // RGBA,
}



use core::mem::size_of;
use crate::{memory::block::{Block, new_block}, virtio::gpu_device::Pixel};
use alloc::{prelude::v1::*};

