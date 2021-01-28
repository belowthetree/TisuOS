//! # Input
//! 处理命令行的输入
//! 2020年12月20日 zg

// use alloc::{prelude::v1::*};
pub const CHAR_LEN : usize = 256;

/// ## 输入信息
/// 只生成一个用于存储和处理输入信息
/// 以循环队列的方式存储、获取输入
struct Input{
    pub read_idx : usize,
    pub write_idx : usize,
    pub list : [char;CHAR_LEN],
}
impl Input {
    pub const fn new() -> Self{
        Input{
            read_idx : 0,
            write_idx : 0,
            list : ['\0';CHAR_LEN],
        }
    }
    pub fn get_input(&mut self){
        let art = Uart::new();
        if let Some(c) = art.get(){
            self.add(c as char);
        }
    }
    pub fn add(&mut self, c : char){
        self.list[self.write_idx] = c;
        self.write_idx = (self.write_idx + 1) % CHAR_LEN;
    }
    pub fn pop(&mut self) ->Option<char> {
        if self.read_idx == self.write_idx {
            None
        }
        else {
            let idx = self.read_idx;
            self.read_idx = (self.read_idx + 1) % CHAR_LEN;
            Some(self.list[idx])
        }
    }
}

static mut INPUT : Option<Input> = None;
// pub static mut HANDLER : Option<Vec::<fn()>> = None;

pub fn init(){
    unsafe {
        INPUT = Some(Input::new());
    }
}

/// ## 驱动全局输入
/// 由 PLIC 驱动
pub fn run(){
    unsafe {
        if let Some(input) = &mut INPUT{
            // 获取输入
            input.get_input();
        }
    }
}

pub fn get()->Option<char> {
    unsafe {
        if let Some(input) = &mut INPUT {
            input.pop()
        }
        else {
            None
        }
    }
}


// use super::shell;
use crate::uart::Uart;