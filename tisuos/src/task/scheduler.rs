//! # 任务调度器
//! 
//! 2021年3月23日 zg

use super::task_manager::{ScheduleMethod, SchedulerOp};


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
    fn schedule<T:super::task_manager::TaskPoolOp>(&mut self, task_pool :&mut T, hartid : usize){
        match self.method {
            ScheduleMethod::Rotation => {
                
            }
        }
    }

    fn switch_method(&mut self, method : super::task_manager::ScheduleMethod) {
    }
}