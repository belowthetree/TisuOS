//! # 桌面
//! 维护一个桌面实例，底层依赖于 Graphic
//! 2020年12月30日 zg

#![allow(dead_code)]

static mut DESKTOP_LOCK : Mutex = Mutex::new();
static mut DESKTOP : Option<Vec<Desktop>> = None;

pub struct Desktop{
    pub valid : bool,
    pub mouse : Mouse,
    pub background : Style,
    pub dock : Dock,
    pub terminal : Vec<Terminal>,
    pub window : Vec<Window>,
    pub cnt : usize,
}

pub struct Window{
}
#[derive(Clone, Copy)]
pub struct ColorChar{
    pub color : Pixel,
    pub c : char,
}

pub struct Terminal{
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
    pub depth : u32,
    pub head_bar : HeadBar,
    pub text : TextContent,
    pub shell : InterShell,
    pub id : usize,
}

pub struct TextContent{
    pub width : u32,
    pub height : u32,
    pub content : Style,
    pub write_x : usize,
    pub write_y : usize,
}

pub struct HeadBar{
    pub background : Style,
    pub close_button : Button,
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
    pub button_width : u32,
    pub close_button_offset : u32,
}

pub struct Dock{
    
}
#[derive(Clone, Copy)]
pub struct Position {
    pub x : u32,
    pub y : u32,
}

impl Position {
    pub fn from_point(point : Point)->Self{
        let x = (point.x * WIDTH as f32) as u32;
        let y = (point.y * HEIGHT as f32) as u32;
        Self{
            x : x,
            y : y,
        }
    }
}

pub fn init(){
    unsafe {
        DESKTOP = Some(Vec::<Desktop>::new());
        register(draw);
    }
}

pub fn run(){
    unsafe {
        loop{
            asm!("wfi"::::"volatile");
            DESKTOP_LOCK.lock();
            if let Some(desk) = &mut DESKTOP{
                for desktop in desk.iter_mut(){
                    desktop.run();
                }
            }
            DESKTOP_LOCK.unlock();
        }
    }
}

pub fn draw(){
    unsafe {
        if let Some(desk) = &mut DESKTOP{
            for desktop in desk.iter_mut(){
                desktop.draw();
            }
        }
    }
}

pub fn register_desktop(desktop : Desktop){
    unsafe {
        DESKTOP_LOCK.lock();
        if let Some(desk) = &mut DESKTOP{
            desk.push(desktop);
        }
        DESKTOP_LOCK.unlock();
    }
}

use alloc::{prelude::v1::*};
// use crate::uart;
use crate::{interact::shell::InterShell, sync::Mutex, virtio::{gpu_device::{HEIGHT, Pixel, WIDTH}, input::input_buffer::{Point, register}}};

use super::{controll::{button::Button, style::style::{Style}}, mouse::Mouse};

