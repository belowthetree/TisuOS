//! # 键盘事件处理
//! 
//! 2021年2月4日 zg

use crate::virtio::input::keyboard::Key;

const BUFFER_SIZE : usize = 128;

#[derive(Clone, Copy)]
pub enum KeyboardEventType {
    None,
    KeyDown,
    KeyUp,
}

#[derive(Copy, Clone)]
pub struct KeyboardEvent {
    pub etype : KeyboardEventType,
    pub key : Key,
}

impl KeyboardEvent {
    pub const fn new(etype : KeyboardEventType, key : Key)->Self {
        Self {
            etype : etype,
            key : key,
        }
    }
}

pub struct Keyboard {
    pub event : [KeyboardEvent;BUFFER_SIZE],
    pub read_idx : usize,
    pub write_idx : usize,
}

impl Keyboard {
    pub fn new()->Self {
        Self {
            event : [KeyboardEvent::new(KeyboardEventType::None,
                Key::None);BUFFER_SIZE],
            read_idx : 0,
            write_idx : 0,
        }
    }
    pub fn get_key_down(&mut self, key : Key) {
        self.add_event(KeyboardEvent::new(KeyboardEventType::KeyDown, key))
    }
    pub fn get_key_release(&mut self, key : Key) {
        self.add_event(KeyboardEvent::new(KeyboardEventType::KeyUp, key))
    }
    pub fn pop_event(&mut self)->Option<KeyboardEvent> {
        if self.write_idx == self.read_idx {
            None
        }
        else {
            let idx = self.read_idx;
            self.read_idx = (self.read_idx + 1) % BUFFER_SIZE;
            Some(self.event[idx])
        }
    }
    fn add_event(&mut self, event : KeyboardEvent) {
        self.event[self.write_idx] = event;
        self.write_idx = (self.write_idx + 1) % BUFFER_SIZE;
    }
}

