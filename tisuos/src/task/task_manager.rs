//! # 任务管理
//! 管理任务系统调度，包含调度算法
//! 2021年3月23日 zg

use core::cmp::Ordering;

use crate::{interrupt::trap::Environment, libs::help::{switch_kernel, switch_user}, memory::page_table::SATP};

use super::{info::{ExecutionInfo, ProgramInfo}, thread::switch_kernel_process};

pub enum ScheduleMethod{
    Rotation,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TaskState {
    Running,
    Waiting,
    Sleeping,
}

/// ## 任务管理器
pub struct TaskManager<T1, T2> {
    scheduler : T1,
    task_pool : T2,
}

/// ## 调度器功能实现
/// 原则上只允许通过任务 ID 进行交互，不得引用、复制以节省处理时间防止同步错误
impl<T1 : SchedulerOp, T2 : TaskPoolOp> TaskManager<T1, T2> {
    pub const fn new(sche : T1, pool : T2)->Self {
        Self {
            scheduler : sche,
            task_pool : pool,
        }
    }

    pub fn start(&mut self, id : usize) {
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.switch_to(&info);
    }

    pub fn schedule(&mut self, env : &mut Environment) {
        let id = self.scheduler.schedule(&mut self.task_pool, env);
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.switch_to(&info);
    }

    pub fn task_exit(&mut self, hartid : usize) {
        self.task_pool.remove_task(hartid).ok();
    }
    /// ### 创建任务并返还任务 ID
    pub fn create_task(&mut self, func : usize, is_kernel : bool)->Option<usize>{
        self.task_pool.create(func, is_kernel)
    }

    pub fn fork_task(&mut self, env : &Environment) {
        let id = self.task_pool.fork(env).unwrap();
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.switch_to(&info);
    }

    pub fn map_code(&mut self, id : usize, virtual_addr : usize, physic_addr : usize) {
        let info = self.task_pool.get_task_prog(id).unwrap();
        let satp = SATP::from(info.satp);
        let pt = satp.get_page_table().unwrap();
        if info.is_kernel {
            pt.map_kernel_code(virtual_addr, physic_addr);
        }
        else {
            pt.map_user_code(virtual_addr, physic_addr);
        }
    }

    pub fn map_data(&mut self, id : usize, virtual_addr : usize, physic_addr : usize) {
        let info = self.task_pool.get_task_prog(id).unwrap();
        let satp = SATP::from(info.satp);
        let pt = satp.get_page_table().unwrap();
        if info.is_kernel {
            pt.map_kernel_data(virtual_addr, physic_addr);
        }
        else {
            pt.map_user_data(virtual_addr, physic_addr);
        }
    }

    pub fn program_exit(&mut self, hartid : usize) {
        let id = self.task_pool.select(|info|{
            info.state == TaskState::Running && info.env.hartid == hartid
        }).unwrap();
        let info = self.task_pool.get_task_exec(id).unwrap();
        let info = self.task_pool.get_task_prog(info.pid).unwrap();
        self.task_pool.remove_task(info.pid);
    }

    fn switch_to(&self, info : &ExecutionInfo) {
        let mut env = info.env;
        if info.is_kernel {
            switch_kernel(&mut env);
        }
        else {
            switch_user(&mut env);
        }
    }
}


/// ## 任务池操作要求
/// 与任务池的操作根据任务号进行，不获取引用，避免同步问题，但提供复制功能
/// 因为这是个多核系统，所以与某个任务交互时必须提供足够的条件（id 或 hartid，state）
pub trait TaskPoolOp {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize>;
    fn set_task_exec(&mut self, id : usize, info : &ExecutionInfo)->Result<(), ()>;
    fn get_task_exec(&mut self, id : usize)->Option<ExecutionInfo>;
    fn get_task_prog(&mut self, id : usize)->Option<ProgramInfo>;
    fn remove_task(&mut self, id : usize)->Result<(), ()>;
    fn remove_program(&mut self, id : usize)->Result<(), ()>;
    fn select<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool;
    fn sort<F>(&mut self, f : F)->usize where F : Fn(&ExecutionInfo, &ExecutionInfo)->Ordering;
    fn fork(&mut self, env : &Environment)->Option<usize>;
}

/// ## 调度器操作要求
/// 算法实现由调度器自身决定
pub trait SchedulerOp{
    /// ### 调度器保存当前任务并选取下一个任务
    fn schedule<T:TaskPoolOp>(&mut self, task_pool :&mut T, env : &mut Environment)->usize;
    fn switch_method(&mut self, method : ScheduleMethod);
}

