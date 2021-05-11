//! # 任务系统
//! 由进程和线程共同支撑
//! 线程作为进程的子集，依赖于进程
//! 同时为了使整体模块化，一切任务交由任务管理器控制，采用接口化编程，任务池、调度器属于可替换模块
//! 
//! 2021年1月1日 zg

mod thread;
mod scheduler;
mod resource;
mod require;

pub mod process;
pub mod task_info;
pub mod task_manager;
pub mod task_pool;

pub static mut TASK_MANAGER : Option<TaskManager<Scheduler, TaskPool>> = None;

pub fn init() {
    process::init();
    thread::init();
    unsafe {
        TASK_MANAGER = Some(TaskManager::new(Scheduler::new(), TaskPool::new()));
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


use self::{scheduler::Scheduler, task_manager::{TaskManager}, task_pool::TaskPool};
