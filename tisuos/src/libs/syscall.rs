//! # 系统调用封装
//! 2021年1月6日 zg

#![allow(dead_code)]
use alloc::prelude::v1::*;
use tisu_driver::Pixel;

use super::str::to_char_slice;

extern "C" {
    fn make_syscall(num : usize, arg1 : usize, arg2 : usize, arg3 : usize, arg4 : usize) -> usize;
    fn make_syscall_long(num : usize, a1 : usize, a2 : usize, a3 : usize, a4 : usize, a5:usize) -> usize;
}

const PROGRAM_EXIT  : usize = 60;
const THREAD_EXIT   : usize = 61;
const FORK          : usize = 57;
const BRANCH        : usize = 7;
const PRINT_TASK    : usize = 5;
const EXEC          : usize = 4;
const MALLOC        : usize = 8;
const WAIT          : usize = 9;
const SET_TIMER     : usize = 10;
const FREE          : usize = 11;
const OPEN          : usize = 12;
const READ          : usize = 13;
const FILE_INFO     : usize = 14;
const WRITE         : usize = 15;
const DRAW_RECT     : usize = 16;
const GET_TIME      : usize = 17;
const GET_KEY_PRESS     : usize = 18;
const GET_KEY_RELEASE   : usize = 19;
const GET_MOUSE_SCROLL  : usize = 20;
const GET_MOUSE_POS     : usize = 21;
const DIRECTORY_INFO    : usize = 22;
const CLOSE             : usize = 23;
const KILL              : usize = 24;
const SHUTDOWN          : usize = 27;
const SLEEP             : usize = 28;

fn syscall(num : usize, arg1 : usize, arg2 : usize, arg3 : usize, arg4 : usize)->usize {
    unsafe {
        make_syscall(num, arg1, arg2, arg3, arg4)
    }
}

fn syscall_long(num : usize, a1 : usize, a2 : usize, a3 : usize, a4 : usize, a5:usize)->usize {
    unsafe {
        make_syscall_long(num, a1, a2, a3, a4, a5)
    }
}

pub fn trigger_software(){
    syscall(1, 0, 0, 0, 0);
}

pub fn trigger_timer(){
    syscall(2, 0, 0, 0, 0);
}

pub fn get_hartid()->usize {
    syscall(3, 0, 0, 0, 0)
}

pub fn sleep() {
    syscall(SLEEP, 0, 0, 0, 0);
}

pub fn shutdown() {
    syscall(SHUTDOWN, 0, 0, 0, 0);
}

pub fn kill(id : usize) {
    syscall(KILL, id, 0, 0, 0);
}

pub fn close(id : usize) {
    syscall(CLOSE, id, 0, 0, 0);
}

pub fn directory_info(path : String)->usize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(DIRECTORY_INFO, p, 0, 0, 0)
}

pub fn get_time()->usize {
    syscall(GET_TIME, 0, 0, 0, 0)
}

pub fn draw_rect(x1:usize, y1:usize, x2:usize, y2:usize, buffer:&[Pixel]) {
    let ptr = buffer as *const [Pixel] as *const Pixel;
    syscall_long(DRAW_RECT, x1, y1, x2, y2, ptr as usize);
}

pub fn write(id : usize, data:&[u8])->usize {
    let addr = data as *const [u8] as *const u8 as usize;
    syscall(WRITE, id, addr, 0, 0)
}

pub fn file_info(path : String)->usize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(FILE_INFO, p, 0, 0, 0)
}

pub fn malloc(size : usize)->usize {
    syscall(MALLOC, size, 0, 0, 0)
}

pub fn fork()->usize{
    syscall(FORK, 0, 0, 0, 0)
}

pub fn exec(path : String)->usize {
    let mut c = Vec::<char>::new();
    for ch in path.as_bytes() {
        c.push(*ch as char);
    }
    let addr = c.as_slice() as *const [char] as *const char as usize;
    syscall(EXEC, addr, path.len(), 0, 0)
}

pub fn list_thread(){
    syscall(PRINT_TASK, 0, 0, 0, 0);
}
#[allow(dead_code)]
pub fn branch(func : usize, a0 : usize, a1 : usize)->usize {
    syscall(BRANCH, func, a0, a1, 0)
}

pub fn wait(target: usize) {
    syscall(WAIT, target, 0, 0, 0);
}

pub fn set_timer(time : usize) {
    syscall(SET_TIMER, time, 0, 0, 0);
}

pub fn free(addr : usize) {
    syscall(FREE, addr, 0, 0, 0);
}

pub fn open(path : String, flag : usize)->isize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(OPEN, p, flag, 0, 0) as isize
}

pub fn read(id : usize, data:&mut [u8])->isize {
    let addr = data as *mut [u8] as *mut u8 as usize;
    syscall(READ, id, addr, data.len(), 0) as isize
}
