//! # 窗口
//! 窗口承包所有内容展示
//! 
//! 2021年2月2日 zg

static mut WINDOW_ID : usize = 1;
pub const WINDOW_SMALLEST_WIDTH : usize = 200;

pub enum WindowEvent {
    None,
    TriggerHidden,
    WindowClose,
    WindowReturn,
}

pub struct Window {
    pub area : Area,
    pub id : usize,
    pub headbar : Headbar,
    pub content : Content,
    pub hidden : bool,
    pub task_id : Option<usize>,
}

impl Window {
    pub fn new(x : usize, y : usize, width : usize, height : usize, ctype : ContentType,
            task_id:Option<usize>)->Self {
        let id = unsafe {
            WINDOW_ID
        };
        unsafe {
            WINDOW_ID += 1;
        }
        let rt = Self {
            area : Area::new(x, y, width, height),
            id,
            headbar : Headbar::new(x, y, width),
            content : Content::new(x, y + HEADBAR_HEIGHT, width, height - HEADBAR_HEIGHT, ctype, task_id),
            hidden : false,
            task_id,
        };
        rt
    }
    /// ### 驱动
    pub fn do_mouse_event(&mut self, event : MouseEvent)->PlaneEvent {
        let mut v = PlaneEvent::None;
        match event.etype {
            MouseEventType::LeftClick => {
                let e = self.headbar.do_mouse_event(event);
                match e {
                    WindowEvent::TriggerHidden => {
                        v = PlaneEvent::WindowSmall(self.id);
                    }
                    WindowEvent::WindowClose => {
                        v = PlaneEvent::WindowClose(self.id);
                    }
                    WindowEvent::WindowReturn => {
                        self.content.return_dir();
                        v = PlaneEvent::WindowInvalid;
                    }
                    _ => {

                    }
                }
            }
            MouseEventType::LeftDoubleClick => {
                let event = self.content.do_mouse_event(event);
                match event {
                    super::content::ContentEvent::Exec(id) =>
                        v = PlaneEvent::CreateTerminal(id),
                    _ => {}
                }
            }
            MouseEventType::Drag => {
                match event.info {
                    MouseEventInfo::Vector(vec) => {
                        self.translate(vec);
                        v = PlaneEvent::WindowInvalid;
                    }
                    _ => {}
                }
            }
            MouseEventType::ScrollDown => {
                self.content.scroll(-10);
                v = PlaneEvent::WindowInvalid;
            }
            MouseEventType::ScrollUp => {
                self.content.scroll(10);
                v = PlaneEvent::WindowInvalid;
            }
            _ => {}
        }
        v
    }

    pub fn do_keyboard_event(&mut self, event : KeyboardEvent)->PlaneEvent {
        if self.content.ctype == ContentType::File {
            return PlaneEvent::None;
        }
        let mut v = PlaneEvent::None;
        match event.etype {
            KeyboardEventType::KeyDown => {
                match event.key.to_char() {
                    Ok(c) => {
                        self.content.get_key(c);
                        v = PlaneEvent::WindowInvalid;
                    }
                    _ => {}
                }
            }
            KeyboardEventType::KeyUp => {}
            _ => {}
        }
        v
    }

    pub fn run(&mut self)->PlaneEvent {
        if let Some(id) = self.task_id {
            let mut e = match self.content.run() {
                super::content::ContentEvent::Invalid => {PlaneEvent::WindowInvalid}
                _ => {PlaneEvent::None}
            };
            let mgr = get_task_mgr().unwrap();
            if mgr.get_task_exec(id).is_none() {
                e = PlaneEvent::WindowClose(self.id);
                self.task_id = None;
            }
            e
        }
        else {
            PlaneEvent::None
        }
    }

    pub fn draw(&mut self) {
        if !self.hidden {
            self.headbar.draw();
            self.content.draw();
        }
    }

    pub fn focus(&mut self, point : Position)->bool {
        !self.hidden && self.area.inside(point)
    }

    pub fn trigger_hidden(&mut self) {
        self.hidden = !self.hidden
    }
}


impl Window {
    fn translate(&mut self, vec : Vector) {
        let x = max(0, self.area.x1 as isize + vec.x) as usize;
        let y = max(0, self.area.y1 as isize + vec.y) as usize;
        let dx = x as isize - self.area.x1 as isize;
        let dy = y as isize - self.area.y1 as isize;
        self.area.x1 = x;
        self.area.y1 = y;
        self.area.x2 = (self.area.x2 as isize + dx) as usize;
        self.area.y2 = (self.area.y2 as isize + dy) as usize;
        let vec = Vector::new(dx, dy);
        self.headbar.translate(vec);
        self.content.translate(vec);
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        if let Some(id) = self.task_id {
            kill(id);
        }
    }
}


use core::cmp::max;

use crate::{libs::{shape::{Area, Position, Vector}, syscall::kill}, task::get_task_mgr};
// use alloc::{prelude::v1::*};
use super::{content::{Content, ContentType}, headbar::{HEADBAR_HEIGHT, Headbar},
    keyboard::{KeyboardEvent, KeyboardEventType}, mouse::{MouseEvent, MouseEventInfo, MouseEventType},
    plane::{PlaneEvent}};

