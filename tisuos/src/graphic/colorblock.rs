//! # 着色块
//! 维护一个内存块，拥有坐标，负责显示功能
//! 
//! 2021年1月31日 zg

#![allow(dead_code)]

const GPU_DEVICE : usize = 0;

pub struct ColorBlock {
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
    pub buffer : Block<Pixel>,
}

/// ## 绘制功能只负责在帧缓冲绘制，并不负责调用 GPU 传输
impl ColorBlock {
    /// ### 以纯色初始化
    pub fn solid_color(x : usize, y : usize, width : usize, height : usize, color : Pixel)->ColorBlock {
        let rt = Self {
            x : x,
            y : y,
            width : width,
            height : height,
            buffer : Block::<Pixel>::new(width * height),
        };
        rt.fill_color(color);
        rt
    }
    /// ### 根据图片建立
    pub fn image(x : usize, y : usize, image : &Image)->Self {
        let buffer = Block::<Pixel>::new(image.width * image.height);
        buffer.copy_from(0, &image.data, 0, image.width * image.height);
        Self {
            x : x,
            y : y,
            width : image.width,
            height : image.height,
            buffer : buffer,
        }
    }
    /// ### 全部填充
    pub fn fill_color(&self, color : Pixel){
        self.buffer.set(0, color, self.width * self.height);
    }
    /// ### 填充某个区域
    pub fn fill_rect(&self, rect : Rect, color : Pixel) {
        if rect.x1 as usize >= self.width || rect.y2 as usize >= self.height {
            return;
        }
        for y in rect.y1 as usize..min(rect.y2 as usize, self.height) {
            let idx = y * self.width + rect.x1 as usize;
            self.buffer.set(idx, color, (rect.x2 - rect.x1) as usize);
        }
    }
    /// ### 绘制指定高度区间
    pub fn draw_area(&self, h1 : usize, h2 : usize) {
        if h1 >= self.height {
            return;
        }
        let rect = Rect {
            x1 : self.x as u32,
            y1 : h1 as u32,
            x2 : (self.x + self.width) as u32,
            y2 : min(h2, self.y + self.height) as u32,
        };

        draw_rect_override(GPU_DEVICE, rect, self.buffer.get_addr() as *mut Pixel);
    }
    /// ### 绘制缓冲全部区域
    pub fn draw_override(&self) {
        let rect = Rect {
            x1 : self.x as u32,
            y1 : self.y as u32,
            x2 : (self.x + self.width) as u32,
            y2 : (self.y + self.height) as u32,
        };

        draw_rect_override(GPU_DEVICE, rect, self.buffer.get_addr() as *mut Pixel);
    }
    /// ### 结合透明度绘制全部区域
    pub fn draw_blend(&self) {
        let rect = Rect {
            x1 : self.x as u32,
            y1 : self.y as u32,
            x2 : (self.x + self.width) as u32,
            y2 : (self.y + self.height) as u32,
        };

        draw_rect_blend(GPU_DEVICE, rect, self.buffer.get_addr() as *mut Pixel);
    }
    /// ### 写字
    pub fn fill_font(&self, c : char, x : usize, y : usize, height : usize, width : usize,
            foreground : Pixel, background : Pixel) {
        let c = c as usize;
        let font = FONT_ASCII[c % 256];
        let mut row = 0;
        
        for yy in y..(y + height) {
            if yy >= self.height as usize {
                break;
            }
            let h = yy * self.width as usize;
            let r = row * FONT_HEIGHT / height;
            let mut col = 0;
            for xx in x..(x + width) {
                if xx >= self.width {
                    break;
                }
                let co = col * FONT_WIDTH / width;
                let v;
                if font[r] & (1 << (7 - co)) != 0 {
                    v = foreground;
                }
                else {
                    v = background;
                }
                self.buffer.set(xx + h, v, 1);
                col += 1;
            }
            row += 1;
        }
    }
    pub fn translate(&mut self, vec : Vector) {
        self.x = max(0, self.x as isize + vec.x) as usize;
        self.y = max(0, self.y as isize + vec.y) as usize;
    }
}

impl ColorBlock {
    pub fn resize(&mut self, width : usize, height : usize) {
        let bak = Block::<Pixel>::new(width * height);
        for y in 0..height {
            let t1 = y * width;
            let t2 = y * self.height / height * self.width;
            for x in 0..width {
                let t = x * self.width / width;
                bak.set(x + t1, self.buffer.get(t + t2).unwrap(), 1);
            }
        }
        self.buffer = bak;
        self.width = width;
        self.height = height;
    }
}


use core::cmp::{max, min};
use crate::{filesystem::image::{image::Image}, libs::{font::{FONT_ASCII, FONT_HEIGHT,
    FONT_WIDTH}, graphic::Pixel, shape::{Rect, Vector}}, memory::block::Block,
    virtio::device::{draw_rect_blend, draw_rect_override}};


