//! # 鼠标事件处理器
//! 
//! 2021年2月4日 zg


const BUFFER_SIZE : usize = 128;
const DOUBLE_INTERVAL : usize = 400; // 双击判定间隔，单位毫秒

pub struct Mouse{
    pub pre_pos : ScalePoint,
    pub cur_pos : ScalePoint,
    pub left : bool,
    pub right : bool,
    pub scroll : isize,
    pub left_click : bool,
    pub left_click_time : usize,
    read_idx : usize,
    write_idx : usize,
    mouse_event : [MouseEvent;BUFFER_SIZE],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MouseEventType{
    None,
    LeftClick,
    LeftDoubleClick,
    RightClick,
    ScrollDown,
    ScrollUp,
    Drag,
}


#[derive(Clone, Copy)]
pub enum MouseEventInfo {
    None,
    Point(Position),
    Key(Key),
    Vector(Vector),
}

#[derive(Clone, Copy)]
pub struct MouseEvent {
    pub etype : MouseEventType,
    pub info : MouseEventInfo,
}

impl MouseEvent {
    pub fn new(etype : MouseEventType, info : MouseEventInfo)->Self {
        Self {
            etype : etype,
            info : info,
        }
    }
}

impl Mouse {
    pub fn new()->Self{
        Self{
            pre_pos : ScalePoint::default(),
            cur_pos : ScalePoint::default(),
            left : false,
            right : false,
            scroll : 0,
            left_click : false,
            left_click_time : 0,
            read_idx : 0,
            write_idx : 0,
            mouse_event : [MouseEvent::new(MouseEventType::None,
                MouseEventInfo::None);BUFFER_SIZE],
        }
    }
    pub fn get_key_down(&mut self, input : Key){
        match input {
            Key::MouseLeft => {
                self.left = true;
                self.add_event(MouseEvent::new(MouseEventType::Drag, MouseEventInfo::Key(input)));
            }
            Key::MouseRight => {
                self.right = true;
            }
            _ => {}
        }
    }
    pub fn get_key_up(&mut self, input : Key){
        match input {
            Key::MouseLeft => {
                let time = get_time();
                let mut double_click = false;
                if self.left_click {
                    if time - self.left_click_time <= DOUBLE_INTERVAL {
                        double_click = true;
                        self.add_event(MouseEvent::new(MouseEventType::LeftDoubleClick,
                            MouseEventInfo::Point(self.get_current_position())));
                    }
                }
                if !double_click{
                    self.add_event(MouseEvent::new(MouseEventType::LeftClick,
                        MouseEventInfo::Point(self.get_current_position())));
                }
                self.left_click = true;
                self.left = false;
                self.left_click_time = time;
            }
            Key::MouseRight => {
                self.add_event(MouseEvent::new(MouseEventType::RightClick,
                    MouseEventInfo::Point(self.get_current_position())));
            }
            _ => {}
        }
    }
    pub fn update_position(&mut self){
        self.pre_pos = self.cur_pos;
        self.cur_pos = get_mouse_position();
        if self.left && self.pre_pos != self.cur_pos {
            let dir = self.cur_pos - self.pre_pos;
            let vec = Vector::from(dir);
            self.add_event(MouseEvent::new(MouseEventType::Drag, MouseEventInfo::Vector(vec)));
        }
    }
    pub fn pop_event(&mut self)->Option<MouseEvent> {
        if self.read_idx == self.write_idx {
            return None;
        }
        let idx = self.read_idx;
        self.read_idx = (self.read_idx + 1) % BUFFER_SIZE;
        Some(self.mouse_event[idx])
    }
    pub fn get_current_position(&self)->Position {
        Position::from_scale_point(self.cur_pos)
    }
    pub fn scroll(&mut self, val : usize){
        if val == 1 {
            self.add_event(MouseEvent::new(MouseEventType::ScrollUp, MouseEventInfo::None))
        }
        else if val == 2 {
            self.add_event(MouseEvent::new(MouseEventType::ScrollDown, MouseEventInfo::None))
        }
    }
    fn add_event(&mut self, event : MouseEvent){
        self.mouse_event[self.write_idx] = event;
        self.write_idx = (self.write_idx + 1) % BUFFER_SIZE;
    }
}


use virtio_input_decoder::Key;

use crate::{Position, ScalePoint, Vector, get_time, libs::input::get_mouse_position};


