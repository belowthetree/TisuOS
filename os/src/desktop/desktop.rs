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

/// ## Bounds
/// 边缘框，用来检测鼠标的相交信息等
pub struct Bounds{
    pub offset_x : u32,
    pub offset_y : u32,
    pub width : u32,
    pub height : u32,
}
pub struct Terminal{
    pub x : u32,
    pub y : u32,
    pub width : u32,
    pub height : u32,
    pub depth : u32,
    pub head_bar : HeadBar,
    pub text : TextContent,
}

pub struct TextContent{
    pub width : u32,
    pub height : u32,
    pub content : Style,
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

pub fn init(){
    unsafe {
        DESKTOP = Some(Vec::<Desktop>::new());
    }
}

pub fn handler(){
    unsafe {
        if let Some(desk) = &mut DESKTOP{
            loop{
                asm!("wfi"::::"volatile");
                DESKTOP_LOCK.lock();
                for desktop in desk.iter_mut(){
                    desktop.run();
                }
                DESKTOP_LOCK.unlock();
            }
        }
    }
}

pub fn register_desktop(desktop : Desktop){
    unsafe {
        if let Some(desk) = &mut DESKTOP{
            DESKTOP_LOCK.lock();
            desk.push(desktop);
            DESKTOP_LOCK.unlock();
        }
    }
}

use alloc::{prelude::v1::*};

use crate::{sync::Mutex, virtio::input::input_buffer::Point};

use super::{controll::{button::Button, style::style::{self, Style}}, mouse::Mouse};

