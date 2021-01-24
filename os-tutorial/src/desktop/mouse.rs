//! # 鼠标
//! 鼠标依赖桌面进行驱动，在内部处理鼠标输入后输出鼠标事件提供给桌面使用
//! 
//! 2021年1月11日 zg

use crate::{interrupt::timer::get_million_time, virtio::{gpu_device::{HEIGHT, WIDTH}, input::{input_buffer::{Point, get_mouse_position}, keyboard::{Key}}}};

const BUFFER_SIZE : usize = 128;
const DOUBLE_INTERVAL : usize = 400; // 双击判定间隔，单位毫秒

pub struct Mouse{
    pub pre_pos : Point,
    pub cur_pos : Point,
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
pub enum MouseEvent{
    None,
    LeftClick,
    LeftDoubleClick,
    RightClick,
    ScrollDown,
    ScrollUp,
    Drag,
}

impl Mouse {
    pub fn new()->Self{
        Self{
            pre_pos : Point::new(),
            cur_pos : Point::new(),
            left : false,
            right : false,
            scroll : 0,
            left_click : false,
            left_click_time : 0,
            read_idx : 0,
            write_idx : 0,
            mouse_event : [MouseEvent::None;BUFFER_SIZE],
        }
    }
    pub fn get_key_down(&mut self, input : Key){
        match input {
            Key::MouseLeft => {
                self.left = true;
                self.add_event(MouseEvent::Drag);
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
                let time = get_million_time();
                let mut double_click = false;
                if self.left_click {
                    if time - self.left_click_time <= DOUBLE_INTERVAL {
                        double_click = true;
                        self.add_event(MouseEvent::LeftDoubleClick);
                    }
                }
                if !double_click{
                    self.add_event(MouseEvent::LeftClick);
                }
                self.left_click = true;
                self.left = false;
                self.left_click_time = time;
            }
            Key::MouseRight => {
                self.add_event(MouseEvent::RightClick);
            }
            _ => {}
        }
    }
    pub fn update_position(&mut self){
        self.pre_pos = self.cur_pos;
        self.cur_pos = get_mouse_position();
        if self.left {
            self.add_event(MouseEvent::Drag);
        }
    }
    pub fn get_move_dir(&self)->(isize, isize){
        let cur = self.get_current_position();
        let pre = self.get_pre_position();
        (cur.0 as isize - pre.0 as isize, cur.1 as isize - pre.1 as isize)
    }
    pub fn get_event(&mut self)->MouseEvent{
        if self.read_idx == self.write_idx {
            return MouseEvent::None;
        }
        let idx = self.read_idx;
        self.read_idx = (self.read_idx + 1) % BUFFER_SIZE;
        self.mouse_event[idx]
    }
    pub fn get_current_position(&self)->(u32, u32){
        let x = (self.cur_pos.x * WIDTH as f32) as u32;
        let y = (self.cur_pos.y * HEIGHT as f32) as u32;
        (x, y)
    }
    pub fn get_pre_position(&self)->(u32, u32){
        let x = (self.pre_pos.x * WIDTH as f32) as u32;
        let y = (self.pre_pos.y * HEIGHT as f32) as u32;
        (x, y)
    }
    pub fn scroll(&mut self, val : u16){
        if val == 1 {
            self.add_event(MouseEvent::ScrollUp)
        }
        else if val == 2 {
            self.add_event(MouseEvent::ScrollDown)
        }
    }
    fn add_event(&mut self, event : MouseEvent){
        self.mouse_event[self.write_idx] = event;
        self.write_idx = (self.write_idx + 1) % BUFFER_SIZE;
    }

}


// use crate::uart;
