//! # 任务系统
//! 由进程和线程共同支撑
//! 线程作为进程的子集，依赖于进程
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

pub mod process;
pub mod thread;
pub mod task_manager;
pub mod scheduler;
pub mod task_pool;
mod info;

static mut PIPE : Option<Vec<info::PipeUnit>> = None;
pub static mut TASK_MANAGER : Option<TaskManager<Scheduler, TaskPool>> = None;

pub fn init() {
    process::init();
    thread::init();
    unsafe {
        TASK_MANAGER = Some(TaskManager::new(Scheduler::new(), TaskPool::new()));
        PIPE = Some(Vec::<info::PipeUnit>::new());
    }
}

/// ## 任务操作接口
/// 试用接口化
pub fn schedule(env : &mut Environment, hartid : usize){
    unsafe {
        if let Some(mgr) = &mut TASK_MANAGER {
            mgr.schedule(env, hartid);
        }
    }
}

pub fn push_pipe(tid : usize, val : usize) {
    unsafe {
        if let Some(pipe) = &mut PIPE {
            for p in pipe.iter_mut() {
                if p.tid == tid {
                    p.push(val);
                    return;
                }
            }
            let mut p = PipeUnit::new(tid);
            p.push(val);
            pipe.push(p);
        }
    }
}

pub fn pop_pipe(tid : usize)->Option<usize> {
    unsafe {
        if let Some(pipe) = &mut PIPE {
            for p in pipe.iter_mut() {
                if p.tid == tid {
                    return p.pop();
                }
            }
        }
    }
    None
}

pub fn delete_pipe(tid : usize) {
    unsafe {
        if let Some(pipe) = &mut PIPE {
            for (idx, p) in pipe.iter_mut().enumerate() {
                if p.tid == tid {
                    pipe.remove(idx);
                    return;
                }
            }
        }
    }
}

use alloc::prelude::v1::*;

use crate::interrupt::trap::Environment;

use self::{info::PipeUnit, scheduler::Scheduler, task_manager::{TaskManager, TaskPoolOp}, task_pool::TaskPool};
