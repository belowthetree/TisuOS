use crate::{interrupt::environment::Environment, memory::{ProgramArea, block::Block}};
use alloc::prelude::v1::*;
use super::task_info::{ExecutionInfo, ProgramInfo};

pub trait TaskPoolOp : TaskPoolBasicOp + TaskComplexOp + TaskResourceOp + TaskScheduleOp {}

/// ## 任务池操作要求
/// 与任务池的操作根据任务号进行，不获取引用，以便模块化
pub trait TaskPoolBasicOp {
    fn create(&mut self, program : ProgramArea, env : &Environment)->Option<usize>;
    fn fork(&mut self, env : &Environment)->Option<usize>;
    fn branch(&mut self, env : &Environment)->Option<usize>;

    fn get_task_exec(&self, id : usize)->Option<ExecutionInfo>;
    fn get_task_prog(&self, id : usize)->Option<ProgramInfo>;

    /// 选取符合条件的多个任务 ID
    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool;
    /// 选取符合条件的第一个任务 ID
    fn find<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool;

    /// 操作所有的任务
    fn operation_all<F>(&mut self,f:F) where F:FnMut(&mut ExecutionInfo);
    /// 操作第一个符合条件的任务
    fn operation_once<F>(&mut self, f:F) where F:FnMut(&ExecutionInfo)->bool;

    fn send_task_msg(&mut self, id : usize, msg : &Block<u8>);

    fn set_task_prog<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut ProgramInfo);
    fn set_task_exec<F>(&mut self, id:usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo);

    fn remove_task(&mut self, id : usize)->Result<(), ()>;
    fn remove_program(&mut self, id : usize)->Result<(), ()>;

    fn print(&self);
}

pub trait TaskComplexOp {
    fn alloc_heap(&mut self, size : usize, id : usize)->(usize, usize);

    fn free_heap(&mut self, addr : usize, id : usize);

    fn virt_to_phy(&self, id:usize, va:usize)->usize;

    /// 等待某个 task 结束，等待者应该退出调度，唤醒操作应该在
    fn wait_task(&mut self, waiter: usize, target: usize);

    /// 检查是否有任务达到了定时器的唤醒时间
    fn check_timer(&mut self, time : usize);

    fn set_timer(&mut self, id : usize, time : usize);

    fn expand_stack(&mut self, id : usize)->Result<(),()>;

    fn join(&mut self, id : usize);
}

pub trait TaskResourceOp {
    fn push_file(&mut self, task_id : usize, file_id:usize);

    fn release_file(&mut self, task_id : usize, file_id:usize);
}

pub trait TaskScheduleOp {
    fn set_priority(&mut self, id:usize, priority : usize);
}
