//! # 任务池
//! 
//! 2021年3月23日 zg


use super::{process::Process, task_manager::{TaskPoolOp, TaskState}, thread::Thread};
use alloc::{collections::VecDeque};

pub struct TaskPool {
    pub process : VecDeque<Process>,
    pub thread : VecDeque<Thread>,
}

impl TaskPool {
    pub fn new()->Self {
        Self{
            process : VecDeque::<Process>::new(),
            thread : VecDeque::<Thread>::new(),
        }
    }
}

impl TaskPoolOp for TaskPool {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize> {
        let p = Process::new(func, is_kernel).unwrap();
        let id = p.pid;
        self.process.push_front(p);
        Some(id)
    }

    fn save_current(&mut self, env : &crate::interrupt::trap::Environment, hartid : usize) {
    }

    fn set_state(&mut self, id : usize, state : TaskState) {
        
    }

    fn switch_to(&self, id : usize) {
    }

    fn delete_current(&mut self, hartid : usize) {
    }

    fn get_task_with_state(&self, hartid : usize, state : TaskState)->usize {
        0
    }
}