//! # 任务信息
//! 
//! 2021年2月6日 zg

use crate::{interrupt::environment::Environment, memory::map::SATP};
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskState {
    Running,
    Waiting,
    Sleeping,
}


#[derive(Debug, Clone, Copy)]
pub struct ExecutionInfo {
    pub priority : usize,
    pub pid : usize,
    pub tid : usize,
    pub state : TaskState,
    pub is_kernel : bool,
    pub is_main : bool,
    pub trigger_time : usize,
    pub stack_top : *mut u8,
    pub env : Environment,
}

#[derive(Clone, Copy, Debug)]
pub struct ProgramInfo {
    pub pid : usize,
    pub satp : SATP,
    pub state : TaskState,
    pub is_kernel : bool,
}


