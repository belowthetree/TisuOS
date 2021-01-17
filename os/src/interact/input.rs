//! # Input
//! 处理全局的输入
//! 2020年12月20日 zg

use alloc::{prelude::v1::*};
pub const CHAR_LEN : usize = 256;
pub struct Input{
    pub idx : usize,
    pub cnt : usize,
    pub list : [char;CHAR_LEN],
}
impl Input {
    pub const fn new() -> Self{
        Input{
            cnt : 0,
            idx : 0,
            list : ['\0';CHAR_LEN],
        }
    }
    pub fn handle_input(&mut self){
        let art = Uart::new();
        if let Some(c) = art.get(){
            self.add(c as char);
        }
    }
    pub fn add(&mut self, c : char){
        self.list[self.idx] = c;
        self.idx = (self.idx + 1) % self.list.len();
        self.cnt = self.cnt.wrapping_add(1);
    }
    pub fn get_front(&self) ->char {
        self.list[(self.list.len() + self.idx - 1) % self.list.len()]
    }
}

pub static mut INPUT : Option<Input> = None;
pub static mut HANDLER : Option<Vec::<fn()>> = None;

pub fn init(){
    unsafe {
        HANDLER = Some(Vec::<fn()>::new());
        INPUT = Some(Input::new());
    }
}

pub fn handler(){
    unsafe {
        if let Some(input) = &mut INPUT{
            input.handle_input();
            if let Some(s) = &mut shell::SHELL{
                for ss in s{
                    ss.handle(input.get_front());
                }
            }
        }
    }
}


use super::shell;
use crate::uart::Uart;