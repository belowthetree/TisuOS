//! 输入缓冲
//! 将所有输入存放进输入循环队列中
//! 输入的添加和取出使用两个互不干扰的变量，在 QUEUE_SIZE 长度的队列中循环存取
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

use alloc::{prelude::v1::*};
use crate::libs::shape::ScalePoint;

const QUEUE_SIZE : usize = 128;
static mut MOUSE_CUR_IDX : usize = 0;
// static mut MOUSE_GET_IDX : usize = 0;
static mut KEY_PRESS_CUR_IDX : usize = 0;
static mut KEY_PRESS_GET_IDX : usize = 0;
pub static mut KEY_RELEASE_CUR_IDX : usize = 0;
pub static mut KEY_RELEASE_GET_IDX : usize = 0;
static mut SCROLL_CUR_IDX : usize = 0;
static mut SCROLL_GET_IDX : usize = 0;
static mut MOUSE_POSITION : [ScalePoint;QUEUE_SIZE] = [ScalePoint::new();QUEUE_SIZE];
static mut KEY_PRESSED : [u16;QUEUE_SIZE] = [0;QUEUE_SIZE];
static mut KEY_RELEASE : [u16;QUEUE_SIZE] = [0;QUEUE_SIZE];
static mut SCROLL : [u16;QUEUE_SIZE] = [0;QUEUE_SIZE]; // 1 是上滑，2 是下滑
static mut DELEGATE : Option<Vec<fn()>> = None;

struct InputBuffer<T> {
    write_idx : usize,
    read_idx : usize,
    buffer : [T;QUEUE_SIZE],
}

impl<T : Copy> InputBuffer<T> {
    pub fn new(val : T)->Self {
        Self{
            write_idx: 0,
            read_idx: 0,
            buffer: [val;QUEUE_SIZE],
        }
    }

    pub fn read(&mut self)->Option<T> {
        if self.read_idx != self.write_idx {
            let idx = self.read_idx;
            self.read_idx = (self.read_idx + 1) % QUEUE_SIZE;
            Some(self.buffer[idx])
        }
        else{
            None
        }
    }

    pub fn write(&mut self, val : T) {
        self.buffer[self.write_idx] = val;
        self.write_idx = (self.write_idx + 1) % QUEUE_SIZE;
    }

    pub fn read_cur(&mut self)->T {
        self.buffer[(self.write_idx + QUEUE_SIZE - 1) % QUEUE_SIZE]
    }
}

pub fn init(){
    unsafe {
        DELEGATE = Some(Vec::<fn()>::new());
    }
}

pub fn get_mouse_position()->ScalePoint{
    unsafe {
        // let idx = MOUSE_GET_IDX;
        // if MOUSE_CUR_IDX != MOUSE_GET_IDX{
        //     MOUSE_GET_IDX = (MOUSE_GET_IDX + 1) % QUEUE_SIZE;
        // }
        MOUSE_POSITION[MOUSE_CUR_IDX]
    }
}

pub fn add_mouse_position(point : ScalePoint){
    unsafe {
        MOUSE_CUR_IDX = (MOUSE_CUR_IDX + 1) % QUEUE_SIZE;
        MOUSE_POSITION[MOUSE_CUR_IDX] = point;
        if let Some(delegate) = &mut DELEGATE {
            for del in delegate {
                del();
            }
        }
    }
}

pub fn get_key_press()->u16{
    unsafe {
        let idx = KEY_PRESS_GET_IDX;
        if KEY_PRESS_GET_IDX != KEY_PRESS_CUR_IDX{
            KEY_PRESS_GET_IDX = (KEY_PRESS_GET_IDX + 1) % QUEUE_SIZE;
        }
        else{
            return 0;
        }
        KEY_PRESSED[idx]
    }
}

pub fn add_key_press(v : u16){
    unsafe {
        KEY_PRESSED[KEY_PRESS_CUR_IDX] = v;
        KEY_PRESS_CUR_IDX = (KEY_PRESS_CUR_IDX + 1) % QUEUE_SIZE;
        if let Some(delegate) = &mut DELEGATE {
            for del in delegate {
                del();
            }
        }
    }
}

pub fn get_key_release()->u16{
    unsafe {
        let idx = KEY_RELEASE_GET_IDX;
        if KEY_RELEASE_GET_IDX != KEY_RELEASE_CUR_IDX{
            KEY_RELEASE_GET_IDX = (KEY_RELEASE_GET_IDX + 1) % QUEUE_SIZE;
        }
        else {
            return 0;
        }
        KEY_RELEASE[idx]
    }
}

pub fn add_key_release(v : u16){
    unsafe {
        KEY_RELEASE[KEY_RELEASE_CUR_IDX] = v;
        KEY_RELEASE_CUR_IDX = (KEY_RELEASE_CUR_IDX + 1) % QUEUE_SIZE;
        if let Some(delegate) = &mut DELEGATE {
            for del in delegate {
                del();
            }
        }
    }
}

pub fn register(f : fn()) {
    unsafe {
        if let Some(delegate) = &mut DELEGATE {
            delegate.push(f);
        }
    }
}

pub fn add_scroll(v : u16){
    unsafe {
        SCROLL[SCROLL_CUR_IDX] = v;
        SCROLL_CUR_IDX = (SCROLL_CUR_IDX + 1) % QUEUE_SIZE;
        if let Some(delegate) = &mut DELEGATE {
            for del in delegate {
                del();
            }
        }
    }
}

pub fn get_scroll()->u16{
    unsafe {
        let idx = SCROLL_GET_IDX;
        if SCROLL_GET_IDX != SCROLL_CUR_IDX{
            SCROLL_GET_IDX = (SCROLL_GET_IDX + 1) % QUEUE_SIZE;
        }
        else {
            return 0;
        }
        SCROLL[idx]
    }
}



// use crate::uart;
