//! # 任务系统
//! 由进程和线程共同支撑
//! 线程作为进程的子集，依赖于进程
//! 同时为了使整体模块化，一切任务交由任务管理器控制，采用接口化编程，进程、线程、调度器属于可替换模块
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

pub mod process;
pub mod thread;
pub mod scheduler;

mod task_info;
pub mod task_manager;
pub mod task_pool;

static mut PIPE : Option<Vec<task_info::PipeUnit>> = None;
pub static mut TASK_MANAGER : Option<TaskManager<Scheduler, TaskPool>> = None;

pub fn init() {
    process::init();
    thread::init();
    unsafe {
        TASK_MANAGER = Some(TaskManager::new(Scheduler::new(), TaskPool::new()));
        PIPE = Some(Vec::<task_info::PipeUnit>::new());
    }
}

pub fn get_task_mgr()->Option<&'static mut TaskManager<Scheduler, TaskPool>> {
    unsafe {
        if let Some(mgr) = &mut TASK_MANAGER {
            Some(mgr)
        }
        else {
            None
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


use self::{task_info::PipeUnit, scheduler::Scheduler, task_manager::{TaskManager}, task_pool::TaskPool};
