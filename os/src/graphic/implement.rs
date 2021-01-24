#![allow(dead_code)]
//! # elment 接口实现
//! 默认使用第一个 GPU 设备
//! 2020年12月29日 zg

const DEVICE_ID : usize = 0;

/// ## Element
/// 

impl Element{
    pub fn new(x : usize, y : usize, width : usize, height : usize)->Self{
        Self{
            x : x,
            y : y,
            width : width,
            height : height,
            content : Block::new(width as usize * height as usize * size_of::<Pixel>()),
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
    pub fn resize(&mut self, width : usize, height : usize){
        self.width = width;
        self.height = height;
        self.content = Block::new(width as usize * height as usize * size_of::<Pixel>());
    }
    pub fn fill_rect(&mut self, rect : Rect, color : Pixel){
        if rect.x1 >= self.width as u32 || rect.y1 >= self.height as u32 {
            return;
        }
        let ptr = self.content.addr as *mut Pixel;
        for y in rect.y1 as usize..min(rect.y2, self.height as u32) as usize {
            let yy = y * self.width as usize;
            for x in rect.x1 as usize..min(rect.x2, self.width as u32) as usize {
                unsafe {
                    ptr.add(x + yy).write_volatile(color);
                }
            }
        }
    }
    pub fn copy(&mut self, other : &Element){
        let count = min(self.height * self.width,
            other.height * other.width) as usize * size_of::<Pixel>();
        unsafe {
            self.content.addr.copy_from(other.content.addr, count);
        }
    }
}

/// ### Draw
/// 

impl Draw for Element {
    fn draw(&self) {
        let rect = Rect{
            x1 : self.x as u32,
            y1 : self.y as u32,
            x2 : (self.x + self.width) as u32,
            y2 : (self.y + self.height) as u32,
        };
        
        draw_rect_override(DEVICE_ID, rect, self.content.addr as *mut Pixel);
    }

    fn fill_font(&mut self, c : usize, x : usize, y : usize, foreground : Pixel, background : Pixel) {
        let font = FONT_ASCII[c % 256];
        let mut row = 0;
        unsafe {
            let ptr = self.content.addr as *mut Pixel;
            for yy in y..(y + FONT_HEIGHT) {
                if yy >= self.height as usize {
                    break;
                }
                let h = yy * self.width as usize;
                let mut col = 0;
                for xx in x..(x + FONT_WIDTH) {
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

    fn draw_offset(&self, rect : Rect) {
        let mut rect = rect;
        if rect.x2 as usize > self.x + self.width {
            rect.x2 = (self.x + self.width) as u32;
        }
        if rect.y2 as usize > self.y + self.height {
            rect.y2 = (self.y + self.height) as u32;
        }
        let r = Rect{
            x1 : self.x as u32,
            y1 : self.y as u32,
            x2 : self.x as u32 + rect.x2 - rect.x1,
            y2 : self.y as u32 + rect.y2 - rect.y1,
        };
        let ptr;
        unsafe {
            ptr = (self.content.addr as *mut Pixel).add(rect.x1 as usize + rect.y1 as usize * self.width);
        }
        draw_rect_override(DEVICE_ID, r, ptr as *mut Pixel);
    }

    fn draw_blend(&self) {
        let rect = Rect{
            x1 : self.x as u32,
            y1 : self.y as u32,
            x2 : (self.x + self.width) as u32,
            y2 : (self.y + self.height) as u32,
        };
        
        draw_rect_blend(DEVICE_ID, rect, self.content.addr as *mut Pixel);
    }

    fn fill_image(&mut self, rect : Rect, image : Image) {
        if rect.x1 as usize >= self.width || rect.y1 as usize >= self.height {
            return;
        }
        let ptr = self.content.addr as *mut Pixel;
        let x1 = rect.x1 as usize;
        let y1 = rect.y1 as usize;
        let x2 = min(self.width, rect.x2 as usize);
        let y2 = min(self.height, rect.y2 as usize);
        let height = y2 - y1;
        let width = x2 - x1;
        let t1 = image.height / height;
        let t2 = image.width / width;
        for y in y1..y2 {
            let yy = (y - y1) * t1;
            let tt = yy * image.width;
            let t = y * self.width;
            for x in x1..x2 {
                unsafe {
                    let xx = (x - x1) * t2;
                    let color1 = *(image.data.addr as *mut Pixel).add(xx + tt);
                    let color2 = *ptr.add(x + t);
                    let rate = color1.a as f32 / 255.0;
                    let rate2 = 1.0 - rate;
                    let color = Pixel{
                        r : (color1.r as f32 * rate) as u8 + (color2.r as f32 * rate2) as u8,
                        g : (color1.g as f32 * rate) as u8 + (color2.g as f32 * rate2) as u8,
                        b : (color1.b as f32 * rate) as u8 + (color2.b as f32 * rate2) as u8,
                        a : (color1.a as f32 * rate) as u8 + (color2.a as f32 * rate2) as u8,
                    };
                    *ptr.add(x + t) = color;
                }
            }
        }
    }
}

/// ### mask Circle
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

/// ### Transform
/// 

impl ElemTranform for Element{
    fn set_position(&mut self, x : usize, y : usize) {
        self.x = x;
        self.y = y;
    }

    fn translate(&mut self, x : isize, y : isize) {
        let mut xx = self.x as isize + x;
        let mut yy = self.y as isize + y;
        if xx < 0{
            xx = 0;
        }
        if yy < 0{
            yy = 0;
        }
        self.x = xx as usize;
        self.y = yy as usize;
    }
}


/// ## Canvas
/// 

impl Canvas {
    pub fn new(x : usize, y : usize, width : usize, height : usize)->Self{
        Self {
            element : Element::new(x, y, width, height),
            page_height : height,
            page_width : width,
            page_num : 1,
            display_offset : 0,
        }
    }
    pub fn detect(&self, point : Position)->bool {
        let xx = self.element.x;
        let yy = self.element.y;
        let width = self.element.width;
        let height = self.element.height;
        let x = point.x;
        let y = point.y;
        xx <= x && yy <= y && xx + width >= x && yy + height >= y
    }
    pub fn scroll(&mut self, offset : i32){
        let idx = self.display_offset as i32 + offset;
        if idx > 0 {
            self.display_offset = idx as usize;
        }
        else if idx + self.page_height as i32 >= self.page_height as i32 * (self.page_num as i32 - 1) {
            self.display_offset = self.page_height as usize * (self.page_num - 1);
        }
        else {
            self.display_offset = 0;
        }
    }
    pub fn fill_rect(&mut self, rect : Rect, color : Pixel){
        if rect.y2 as usize >= self.page_num * self.page_height {
            self.add_page(2);
        }
        let rect = Rect {
            x1 : rect.x1,
            y1 : rect.y1,
            x2 : rect.x2,
            y2 : rect.y2,
        };
        self.element.fill_rect(rect, color);
    }
    pub fn add_page(&mut self, n : usize){
        let mut new_ele = Element::new(self.element.x, self.element.y,
            self.element.width, self.element.height + n * self.page_height);
        new_ele.copy(&self.element);
        self.element = new_ele;
        self.page_num += n;
    }
    pub fn fill(&mut self, color : Pixel){
        self.element.fill(color);
    }
}

/// ### Draw
/// 
impl Draw for Canvas {
    fn draw(&self) {
        let rect = Rect {
            x1 : 0,
            y1 : self.display_offset as u32,
            x2 : self.page_width as u32,
            y2 : self.display_offset as u32 + self.page_height as u32
        };
        self.element.draw_offset(rect)
    }
    fn draw_offset(&self, rect : Rect) {
        let mut rect = rect;
        rect.y1 += self.display_offset as u32;
        rect.y2 += self.display_offset as u32 + self.page_height as u32;
        self.element.draw_offset(rect);
    }

    fn fill_font(&mut self, c : usize, x : usize, y : usize, foreground : Pixel, background : Pixel) {
        // println!("fill font y {}", y);
        if y + FONT_HEIGHT >= self.page_num * self.page_height as usize {
            self.add_page(self.page_num);
            // println!("add page");
        }
        if y + FONT_HEIGHT > self.page_height as usize{
            self.display_offset = y + FONT_HEIGHT - self.page_height as usize;
        }
        else {
            self.display_offset = 0;
        }
        self.element.fill_font(c, x, y, foreground, background);
    }
    /// 画布暂时不支持透明绘制
    fn draw_blend(&self) {
    }

    fn fill_image(&mut self, rect : Rect, image : Image) {
        if rect.y2 as usize >= self.page_num * self.page_height {
            self.add_page(2);
        }
        self.element.fill_image(rect, image);
    }
}

/// ## Transform
/// 
impl ElemTranform for Canvas{
    fn set_position(&mut self, x : usize, y : usize) {
        self.element.set_position(x, y);
    }

    fn translate(&mut self, x : isize, y : isize) {
        self.element.translate(x, y);
    }
}

// /// 调用 GPU 刷新显示
// fn flush_gpu(){
//     gpu_device::flush(DEVICE_ID);
// }


use core::{cmp::min, intrinsics::sqrtf64, mem::size_of};

use gpu_device::{Pixel, Rect, draw_rect_blend};

use crate::{desktop::{desktop::Position, implement::{FONT_HEIGHT, FONT_WIDTH}}, libs::font::FONT_ASCII, memory::block::Block, virtio::gpu_device::{draw_rect_override}};
use crate::virtio::gpu_device;
use crate::filesystem::image::image::Image;
use super::{canvas::Canvas, element::{Draw, Element}, mask::Circle, transform::ElemTranform};
