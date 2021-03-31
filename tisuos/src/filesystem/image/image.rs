//! 图片
//! 所有图片格式最终转换成此接口
//! 
//! 2020年12月31日 zg

static mut GLOBAL_IMAGE : Option<Vec<GlobalImage>> = None;

pub fn init() {
    unsafe {
        GLOBAL_IMAGE = Some(Vec::<GlobalImage>::new());
    }
}

pub struct GlobalImage {
    pub path : String,
    pub img : Image,
}

impl GlobalImage {
    pub fn new(path : &String, img : Image)->Self {
        Self {
            path : path.clone(),
            img : img,
        }
    }
}

pub struct Image{
    pub width : usize,
    pub height : usize,
    pub format : Format,
    pub data : Block<Pixel>,
}

impl Clone for Image {
    fn clone(&self) -> Self {
        let rt = Image::new(self.width, self.height, Format::RGB);
        rt.copy(self);
        rt
    }
}


/// ## 默认以 Pixel 为单位读写
impl Image {
    /// ### 从全局图片库中查找图片，如果不存在则创建一个
    pub fn request(path : &String, width : usize, height : usize)->Option<Image> {
        unsafe {
            if let Some(gi) = &mut GLOBAL_IMAGE {
                for gimg in gi.iter_mut() {
                    let img = &gimg.img;
                    if width == img.width && height == img.height && *path == gimg.path {
                        return Some(img.clone());
                    } 
                }
                let mut img = Image::open(path).unwrap();
                img.resize(width, height);
                let rt = img.clone();
                gi.push(GlobalImage::new(path, img));
                Some(rt)
            }
            else {
                None
            }
        }
    }
    pub fn copy(&self, img : &Image) {
        self.data.copy_from(0, &img.data, 0, self.data.size);
    }
    pub fn open(path : &String)->Option<Self> {
        let mut file = File::open(path).unwrap();
        file.open_flag(OpenFlag::Read.val()).unwrap();
        generate_image(file)
    }
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
        let count = self.width * self.height;
        let b = Block::<Pixel>::new(count);
        b.copy_from(0, &self.data, 0, count);
        for y in 0..self.height {
            self.data.copy_from(y * self.width,
            &b, count - (y + 1) * self.width, self.width);
        }
    }
    pub fn resize(&mut self, width : usize, height : usize) {
        let tm = Block::<Pixel>::new(width * height);
        // let ptr1 = self.data.get_addr();
        // let ptr2 = tm.get_addr();
        for y in 0..height {
            let yy = y * self.height / height;
            let t1 = y * width;
            let t2 = yy * self.width;
            for x in 0..width {
                let xx = x * self.width / width;
                tm.set(x + t1, self.data.get(xx + t2).unwrap(), 1);
                // let v = *(ptr1.add(xx + t2));
                // ptr2.add(x + t1).write_volatile(v);
                // tm.set(x + t1, v, 1);
            
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

use crate::{filesystem::filetree::file::{File, OpenFlag}};
use crate::{libs::graphic::Pixel, memory::block::{Block}};
use alloc::prelude::v1::*;
use super::bmp::generate_image;

