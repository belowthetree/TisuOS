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
    fn schedule<T:super::require::TaskPoolBasicOp>(&mut self, task_pool :&mut T)->Option<usize> {
        let next;
        match self.method {
            ScheduleMethod::Rotation => {
                let mut id = None;
                let mut mx = 0;
                task_pool.operation_all(|info|{
                    if info.state == TaskState::Waiting {
                        if mx <= info.priority {
                            id = Some(info.tid);
                            mx = info.priority;
                            info.priority += 1;
                        }
                    }
                });
                next = id;
                if let Some(next) = next {
                    task_pool.set_task_exec(next, |info|{
                        info.priority = 0;
                    }).unwrap();
                }
            }
        }
        next
    }

    fn switch_method(&mut self, _ : super::task_manager::ScheduleMethod) {
    }
}