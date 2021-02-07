//! # 快捷功能坞
//! 
//! 2021年2月5日 zg


const DOCK_HEIGHT : usize = 60;
const DOCK_WIDTH : usize = 300;
const DOCK_ITEM_PADDING : usize = 5;

#[derive(Clone, Copy)]
pub enum DockFuncType {
    OpenFileWindow,
    OpenTerminal,
    TriggerFileWindow(usize),
    TriggerTerminal(usize),
}

pub struct DockFunc {
    pub button : Button,
    pub ftype : DockFuncType,
}

impl DockFunc {
    pub fn new(ftype : DockFuncType)->Self {
        let image;
        match ftype {
            DockFuncType::OpenFileWindow | DockFuncType::TriggerFileWindow(_) => {
                image = Image::open(&"0/img/folder.bmp".to_string()).unwrap();
            }
            DockFuncType::TriggerTerminal(_) | DockFuncType::OpenTerminal => {
                image = Image::open(&"0/img/terminal_color.bmp".to_string()).unwrap();
            }
        }

        Self {
            button : Button::image(0, 0, DOCK_HEIGHT, DOCK_HEIGHT, &image),
            ftype : ftype,
        }
    }
}

pub struct Dock {
    pub area : Area,
    pub background : ColorBlock,
    pub func : Vec<DockFunc>,
}

impl Dock {
    pub fn new()->Self {
        let x = (WIDTH - DOCK_WIDTH) / 2;
        let y = HEIGHT - DOCK_HEIGHT;
        let h = DOCK_HEIGHT / 2;
        let color = Pixel::new(255, 255, 255, 55);
        let mut rt = Self {
            area : Area::new(x, y, DOCK_WIDTH, DOCK_HEIGHT),
            background : ColorBlock::solid_color(x, y + h, DOCK_WIDTH, h, color),
            func : Vec::<DockFunc>::new(),
        };
        rt.add_func(DockFuncType::OpenFileWindow);
        rt.add_func(DockFuncType::OpenTerminal);
        rt
    }
    pub fn draw(&self) {
        self.background.draw_blend();
        for f in self.func.iter() {
            f.button.draw_blend();
        }
    }
    pub fn add_func(&mut self, ftype : DockFuncType) {
        let func = DockFunc::new(ftype);
        self.func.push(func);
        self.relist();
    }
    fn relist(&mut self) {
        let len = self.func.len() * DOCK_HEIGHT + (self.func.len() - 1) * DOCK_ITEM_PADDING;
        let mut x = WIDTH / 2 - len / 2;
        let width;
        if len > WIDTH {
            width = WIDTH / self.func.len();
            for func in self.func.iter_mut() {
                func.button.resize(width, width);
            }
            // x = 0;
        }
        else {
            width = DOCK_HEIGHT;
        }

        for func in self.func.iter_mut() {
            if func.button.background.width != width {
                func.button.resize(width, width);
            }
            func.button.set_position(Position::new(x, HEIGHT - width));
            x += width + DOCK_ITEM_PADDING;
        }
    }
    pub fn trigger(&mut self, point : Position)->Option<DockFuncType> {
        for btn in self.func.iter() {
            if btn.button.detect(point) {
                return Some(btn.ftype);
            }
        }
        None
    }
    pub fn do_mouse_event(&mut self, event : MouseEvent)->PlaneEvent {
        match event.etype {
            super::mouse::MouseEventType::LeftClick => {
                match event.info {
                    super::mouse::MouseEventInfo::Point(point) => {
                        if let Some(ftype) = self.trigger(point) {
                            match ftype {
                                DockFuncType::OpenFileWindow => { return PlaneEvent::CreateFileWindow; }
                                DockFuncType::OpenTerminal => { return PlaneEvent::CreateTerminal; }
                                DockFuncType::TriggerFileWindow(id) => { return PlaneEvent::WindowSmall(id) }
                                DockFuncType::TriggerTerminal(id) => { return PlaneEvent::WindowSmall(id) }
                            }
                        }
                        else {
                            PlaneEvent::None
                        }
                    }
                    _ => {PlaneEvent::None}
                }
            }
            _ => {
                PlaneEvent::None
            }
        }
    }
    pub fn remove_func(&mut self, id : usize) {
        for (idx, func) in self.func.iter_mut().enumerate() {
            match func.ftype {
                DockFuncType::OpenFileWindow => {}
                DockFuncType::OpenTerminal => {}
                DockFuncType::TriggerFileWindow(idd) => {
                    if id == idd {
                        self.func.remove(idx);
                        break;
                    }
                }
                DockFuncType::TriggerTerminal(idd) => {
                    if id == idd {
                        self.func.remove(idx);
                        break;
                    }
                }
            }
        }
        self.relist();
    }
}


// use crate::uart;
use crate::{filesystem::image::image::Image, graphic::colorblock::ColorBlock, libs::{graphic::Pixel, shape::{Area, Position}}, virtio::gpu_device::{HEIGHT, WIDTH}};
use alloc::prelude::v1::*;
use super::{button::Button, mouse::MouseEvent, plane::PlaneEvent};

