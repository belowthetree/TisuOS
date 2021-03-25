//! # 任务调度器
//! 
//! 2021年3月23日 zg

use crate::interrupt::trap::Environment;

use super::{info::ExecutionInfo, task_manager::{ScheduleMethod, SchedulerOp, TaskState}};


pub struct Scheduler{
    pub method : ScheduleMethod,
}

impl Scheduler {
    pub const fn new()->Self{
        Self {
            method : ScheduleMethod::Rotation,
        }
    }
}

impl SchedulerOp for Scheduler {
    fn schedule<T:super::task_manager::TaskPoolOp>(&mut self, task_pool :&mut T,
            env : &mut Environment, hartid : usize)->usize {
        let next;
        match self.method {
            ScheduleMethod::Rotation => {
                let cur = task_pool.select(|info|{
                    if info.state == TaskState::Running && info.env.hartid == hartid {
                        true
                    }
                    else {
                        false
                    }
                }).unwrap();
                let mut info = task_pool.get_task_exec(cur).unwrap();
                info.env = *env;
                info.state = TaskState::Waiting;
                next = task_pool.select(|info| {
                    if info.state == TaskState::Waiting {
                        true
                    }
                    else {
                        false
                    }
                }).unwrap();
                task_pool.set_task_exec(cur, &info).ok();
            }
        }
        next
    }

    fn switch_method(&mut self, method : super::task_manager::ScheduleMethod) {
    }
}