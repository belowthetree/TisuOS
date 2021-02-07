//! # 任务信息
//! 
//! 2021年2月6日 zg

use crate::interrupt::trap::Environment;

use super::thread::Thread;

pub struct Info {
    pub pid : usize,
    pub tid : usize,
    pub is_kernel : bool,
    pub stack_top : *mut u8,
    pub env : Environment,
}

impl Info {
    pub fn from_thread(thread : &Thread)->Self {
        Self {
            pid : thread.pid,
            tid : thread.tid,
            is_kernel : thread.is_kernel,
            stack_top : thread.stack_top,
            env : thread.env,
        }
    }
}