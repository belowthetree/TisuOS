use super::{str::to_char_slice};
use alloc::prelude::v1::*;
use tisu_driver::Pixel;
global_asm!(include_str!("../func.S"));


const FORK          : usize = 57;
#[allow(dead_code)]
const PRINT_TASK    : usize = 5;
#[allow(dead_code)]
const EXEC          : usize = 4;
const BRANCH        : usize = 7;
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

extern  "C" {
    fn env_call_tuple(num:usize, a0 : usize, a1: usize, a2: usize)->(usize, usize);
    fn env_call(num:usize, a0 : usize, a1: usize, a2: usize)->usize;
    fn env_call_long(num:usize, a0:usize, a1:usize, a2:usize, a3:usize, a4:usize,a5:usize)->usize;
}

pub fn directory_info(path : String)->usize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(DIRECTORY_INFO, p, 0, 0)
}

pub fn sys_mouse_position()->(usize, usize) {
    unsafe {
        env_call_tuple(GET_MOUSE_POS, 0, 0, 0)
    }
}

pub fn sys_mouse_scroll()->usize {
    syscall(GET_MOUSE_SCROLL, 0, 0, 0)
}

pub fn sys_key_release()->usize {
    syscall(GET_KEY_RELEASE, 0, 0, 0)
}

pub fn sys_key_press()->usize {
    syscall(GET_KEY_PRESS, 0, 0, 0)
}

pub fn get_time()->usize {
    syscall(GET_TIME, 0, 0, 0)
}

pub fn draw_rect(x1:usize, y1:usize, x2:usize, y2:usize, buffer:&[Pixel]) {
    let ptr = buffer as *const [Pixel] as *const Pixel;
    syscall_long(DRAW_RECT, x1, y1, x2, y2, ptr as usize, 0);
}

pub fn write(id : usize, data:&[u8])->usize {
    let addr = data as *const [u8] as *const u8 as usize;
    syscall(WRITE, id, addr, data.len())
}

pub fn file_info(path : String)->usize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(FILE_INFO, p, 0, 0)
}

pub fn read(id : usize, data:&mut [u8])->isize {
    let addr = data as *mut [u8] as *mut u8 as usize;
    syscall(READ, id, addr, data.len()) as isize
}

pub fn open(path : String, flag : usize)->isize {
    let path = to_char_slice(&path);
    let p = path.as_slice() as *const [char] as *const char as usize;
    syscall(OPEN, p, flag, 0) as isize
}

pub fn syscall_test() {
    syscall(1, 0, 0, 0);
}

pub fn syscall_test2() {
    syscall(2, 0, 0, 0);
}

/// 类似Linux的fork，从当前地址创建新的线程
pub fn fork()->usize {
    syscall(FORK, 0, 0, 0)
}

/// 分支执行，从指定函数创建新的线程
pub fn branch(entry : usize)->usize {
    syscall(BRANCH, entry, 0, 0)
}

/// 申请内存，采用内存池实现
pub fn malloc(size : usize)->usize {
    syscall(MALLOC, size, 0, 0)
}

/// 等待某个线程死亡
pub fn wait(id : usize) {
    syscall(WAIT, id, 0, 0);
}

/// 休眠当前线程，时间单位微秒
pub fn set_timer(time : usize) {
    syscall(SET_TIMER, time, 0, 0);
}

pub fn free(addr : usize) {
    syscall(FREE, addr, 0, 0);
}


fn syscall(num:usize, a0: usize, a1: usize, a2: usize)->usize {
    unsafe {
        env_call(num, a0, a1, a2)
    }
}

fn syscall_long(num:usize, a0: usize, a1: usize, a2: usize, a3:usize,a4:usize,a5:usize)->usize {
    unsafe {
        env_call_long(num, a0, a1, a2, a3, a4, a5)
    }
}