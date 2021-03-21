//! # 任务信息
//! 
//! 2021年2月6日 zg

use crate::{interrupt::trap::Environment, sync::Mutex};
use alloc::{collections::VecDeque};
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

pub struct PipeUnit {
    pub tid : usize,
    pub val : VecDeque<usize>,
    pub lock : Mutex,
}

impl PipeUnit {
    pub fn new(tid : usize)->Self {
        Self {
            tid : tid,
            val : VecDeque::<usize>::new(),
            lock : Mutex::new(),
        }
    }
    pub fn push(&mut self, val : usize) {
        self.lock.lock();
        self.val.push_back(val);
        self.lock.unlock();
    }
    pub fn pop(&mut self)->Option<usize> {
        self.lock.lock();
        let rt = self.val.pop_front();
        self.lock.unlock();
        rt
    }
}

