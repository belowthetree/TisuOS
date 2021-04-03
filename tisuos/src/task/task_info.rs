//! # 任务信息
//! 
//! 2021年2月6日 zg

use crate::{interrupt::trap::Environment, sync::mutex::Mutex};
use alloc::{collections::VecDeque};


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskState {
    Running,
    Waiting,
    Sleeping,
}


#[derive(Debug)]
pub struct ExecutionInfo {
    pub pid : usize,
    pub tid : usize,
    pub state : TaskState,
    pub is_kernel : bool,
    pub stack_top : *mut u8,
    pub env : Environment,
}

#[derive(Clone, Copy)]
pub struct ProgramInfo {
    pub pid : usize,
    pub satp : usize,
    pub state : TaskState,
    pub is_kernel : bool,
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

