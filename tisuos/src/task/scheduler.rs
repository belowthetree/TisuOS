//! # 任务调度器
//! 
//! 2021年3月23日 zg


use super::{task_manager::{ScheduleMethod, SchedulerOp}, task_info::TaskState};


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
    fn schedule<T:super::task_manager::TaskPoolBasicOp>(&mut self, task_pool :&mut T)->Option<usize> {
        let next;
        match self.method {
            ScheduleMethod::Rotation => {
                next = task_pool.find(|info| {
                    info.state == TaskState::Waiting
                });
            }
        }
        next
    }

    fn switch_method(&mut self, _ : super::task_manager::ScheduleMethod) {
    }
}