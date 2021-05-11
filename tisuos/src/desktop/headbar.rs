//! # 标题栏
//! 窗口通用标题栏
//! 
//! 2021年2月2日 zg

const HEADBAR_COLOR : Pixel = Pixel::grey();
const BUTTON_WIDTH : usize = 30;
pub const HEADBAR_HEIGHT : usize = 20;

pub struct Headbar {
    pub area : Area,
    pub background : ColorBlock,
    pub close_button : Button,
    pub small_button : Button,
    pub return_button : Button,
}

impl Headbar {
    pub fn new(x : usize, y : usize, width : usize)->Self {
        let width = max(width, WINDOW_SMALLEST_WIDTH);
        Self {
            area : Area::new(x, y, width, HEADBAR_HEIGHT),
            background : ColorBlock::solid_color(x, y, width, HEADBAR_HEIGHT, HEADBAR_COLOR),
            close_button : Button::new(x + width - BUTTON_WIDTH,
                y, BUTTON_WIDTH, HEADBAR_HEIGHT, Pixel::red()),
            small_button : Button::new(x + width - BUTTON_WIDTH * 2,
                y, BUTTON_WIDTH, HEADBAR_HEIGHT, Pixel::shallow_grey()),
            return_button : Button::new(x, y, BUTTON_WIDTH, HEADBAR_HEIGHT, Pixel::shallow_grey()),
        }
    }
    pub fn draw(&mut self) {
        self.background.draw_override();
        self.close_button.draw();
        self.small_button.draw();
        self.return_button.draw();
    }
    pub fn do_mouse_event(&mut self, event : MouseEvent)->WindowEvent {
        match event.etype {
            super::mouse::MouseEventType::LeftClick => {
                if let MouseEventInfo::Point(point) = event.info {
                    if self.close_button.detect(point) {
                        WindowEvent::WindowClose
                    }
                    else if self.small_button.detect(point) {
                        WindowEvent::TriggerHidden
                    }
                    else if self.return_button.detect(point) {
                        WindowEvent::WindowReturn
                    }
                    else {
                        WindowEvent::None
                    }
                }
                else {
                    WindowEvent::None
                }
            }
            _ => { WindowEvent::None }
        }
    }
}

impl Headbar {
    pub fn translate(&mut self, vec : Vector) {
        let x = max(0, self.area.x1 as isize + vec.x);
        let y = max(0, self.area.y1 as isize + vec.y);
        self.area.x1 = x as usize;
        self.area.y1 = y as usize;
        self.background.x = self.area.x1;
        self.background.y = self.area.y1;
        self.close_button.translate(vec);
        self.small_button.translate(vec);
        self.return_button.translate(vec);
    }
}


use core::cmp::max;

use crate::{graphic::colorblock::ColorBlock, libs::{graphic::Pixel, shape::{Area, Vector}}};

use super::{button::Button, mouse::{MouseEvent, MouseEventInfo}, window::{WINDOW_SMALLEST_WIDTH, WindowEvent}};


