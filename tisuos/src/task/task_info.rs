//! # 任务信息
//! 
//! 2021年2月6日 zg

use crate::{interrupt::trap::Environment, sync::Mutex};
use alloc::{collections::VecDeque};
use super::{process::Process, task_manager::TaskState, thread::Thread};

#[derive(Debug)]
pub struct ExecutionInfo {
    pub pid : usize,
    pub tid : usize,
    pub state : TaskState,
    pub is_kernel : bool,
    pub stack_top : *mut u8,
    pub env : Environment,
}

pub struct ProgramInfo {
    pub pid : usize,
    pub satp : usize,
    pub state : TaskState,
    pub is_kernel : bool,
}

impl ExecutionInfo {
    pub fn default()->Self {
        Self {
            pid: 0,
            tid: 0,
            state: TaskState::Waiting,
            is_kernel: false,
            stack_top: 0 as *mut u8,
            env: Environment::new(),
        }
    }
    
    pub fn from_thread(thread : &Thread)->Self {
        Self {
            pid : thread.pid,
            tid : thread.tid,
            state : thread.state.to_task_state(),
            is_kernel : thread.is_kernel,
            stack_top : thread.stack_top,
            env : thread.env,
        }
    }

    pub fn from_env(env : &Environment)->Self {
        let mut rt = Self::default();
        rt.env = *env;
        rt
    }
}

impl ProgramInfo {
    pub fn default()->Self {
        Self {
            pid: 0,
            satp : 0,
            state: TaskState::Waiting,
            is_kernel: false,
        }
    }
    
    pub fn from_program(process : &Process)->Self {
        Self {
            pid : process.pid,
            satp : process.satp,
            state : process.state.to_task_state(),
            is_kernel : process.is_kernel,
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

