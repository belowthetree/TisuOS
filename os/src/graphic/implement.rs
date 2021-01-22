#![allow(dead_code)]
//! # elment 接口实现
//! 默认使用第一个 GPU 设备
//! 2020年12月29日 zg

const DEVICE_ID : usize = 0;

impl Element{
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        Self{
            x : x,
            y : y,
            width : width,
            height : height,
            content : new_block(width as usize * height as usize * size_of::<Pixel>()),
        }
    }
    pub fn fill(&mut self, color : Pixel){
        let ptr = self.content.addr as *mut Pixel;
        // println!("before fill");
        for y in 0..self.width as usize{
            for x in (0..(self.height * self.width) as usize).step_by(self.width as usize){
                unsafe {
                    ptr.add(x + y).write_volatile(color);
                }
            }
        }
        // println!("after fill");
    }
    pub fn resize(&mut self, width : u32, height : u32){
        self.width = width;
        self.height = height;
        self.content = new_block(width as usize * height as usize * size_of::<Pixel>());
    }
}

///
/// ## Draw
/// 

impl Draw for Element {
    fn draw(&self) {
        let rect = Rect{
            x1 : self.x,
            y1 : self.y,
            x2 : self.x + self.width,
            y2 : self.y + self.height,
        };
        
        draw_rect_override(DEVICE_ID, rect, self.content.addr as *mut Pixel);
    }

    fn fill_font(&self, c : usize, x : usize, y : usize, foreground : Pixel, background : Pixel) {
        let font = FONT_ASCII[c % 256];
        let mut row = 0;
        unsafe {
            let ptr = self.content.addr as *mut Pixel;
            for yy in y..(y + 16) {
                if yy >= self.height as usize {
                    break;
                }
                let h = yy * self.width as usize;
                let mut col = 0;
                for xx in x..(x + 8) {
                    let v;
                    // println!("col {}", col);
                    if font[row] & (1 << (7 - col)) != 0 {
                        v = foreground;
                    }
                    else {
                        v = background;
                    }
                    ptr.add(xx + h).write(v);
                    col += 1;
                }
                row += 1;
            }
            
        }
    }

    fn draw_area(&self, rect : Rect) {
        let mut rect = rect;
        if rect.x2 > self.x + self.width {
            rect.x2 = self.x + self.width;
        }
        if rect.y2 > self.y + self.height {
            rect.y2 = self.y + self.height;
        }
        let r = Rect{
            x1 : self.x + rect.x1,
            y1 : self.y + rect.y1,
            x2 : self.x + rect.x2 - rect.x1,
            y2 : self.y + rect.y2 - rect.y1,
        };
        let ptr;
        unsafe {
            ptr = (self.content.addr as *mut Pixel).add((rect.x1 + rect.y1 * self.width) as usize);
        }
        draw_rect_override(DEVICE_ID, r, ptr as *mut Pixel);
    }
}

///
/// mask Circle
/// 

impl Circle for Element{
    fn mask(&mut self, radius : usize) {
        assert!(radius * 2 < self.width as usize && radius * 2 < self.height as usize);
        let ptr = self.content.addr as *mut Pixel;
        let center = (self.width as usize + 1) / 2;
        let wid = self.width as usize;
        let r2 = radius * radius;
        for y in 0..radius{
            let mut len = unsafe {sqrtf64((r2 - y * y) as f64)} as usize;
            if len > radius {
                len = radius;
            }
            for x in len..center{
                unsafe {
                    let mut color = ptr.add((center + y + 1) * wid + center + x).read_volatile();
                    color.a = 0;
                    ptr.add((center + y + 1) * wid + center + x).write_volatile(color);
                    let mut color = ptr.add((center + y + 1) * wid + center - x).read_volatile();
                    color.a = 0;
                    ptr.add((center + y + 1) * wid + center - x).write_volatile(color);
                    let mut color = ptr.add((center - y - 1) * wid + center + x).read_volatile();
                    color.a = 0;
                    ptr.add((center - y - 1) * wid + center + x).write_volatile(color);
                    let mut color = ptr.add((center - y - 1) * wid + center - x).read_volatile();
                    color.a = 0;
                    ptr.add((center - y - 1) * wid + center - x).write_volatile(color);
                }
            }
        }
    }

    fn mask_inverse(&mut self, radius : usize) {
        assert!(radius * 2 < self.width as usize && radius * 2 < self.height as usize);
        let ptr = self.content.addr as *mut Pixel;
        let center = (self.width as usize + 1) / 2;
        let wid = self.width as usize;
        let r2 = radius * radius;
        for y in 0..radius{
            let mut len = unsafe {sqrtf64((r2 - y * y) as f64)} as usize;
            if len > radius {
                len = radius;
            }
            for x in 0..len{
                unsafe {
                    let mut color = ptr.add((center + y + 1) * wid + center + x).read_volatile();
                    color.a = 0;
                    ptr.add((center + y + 1) * wid + center + x).write_volatile(color);
                    let mut color = ptr.add((center + y + 1) * wid + center - x).read_volatile();
                    color.a = 0;
                    ptr.add((center + y + 1) * wid + center - x).write_volatile(color);
                    let mut color = ptr.add((center - y - 1) * wid + center + x).read_volatile();
                    color.a = 0;
                    ptr.add((center - y - 1) * wid + center + x).write_volatile(color);
                    let mut color = ptr.add((center - y - 1) * wid + center - x).read_volatile();
                    color.a = 0;
                    ptr.add((center - y - 1) * wid + center - x).write_volatile(color);
                }
            }
        }
    }
}

///
/// Transform
/// 

impl ElemTranform for Element{
    fn set_position(&mut self, x : u32, y : u32) {
        println!("set x {}, y {}", x, y);
        self.x = x;
        self.y = y;
    }
}



// /// 调用 GPU 刷新显示
// fn flush_gpu(){
//     gpu_device::flush(DEVICE_ID);
// }


use core::{intrinsics::sqrtf64, mem::size_of};

use gpu_device::{Pixel, Rect};

use crate::{libs::font::FONT_ASCII, memory::block::new_block, uart, virtio::gpu_device::{draw_rect_override}};
use crate::virtio::gpu_device;

use super::{element::{Draw, Element}, mask::Circle, transform::ElemTranform};
