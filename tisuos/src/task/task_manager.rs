//! # 任务管理
//! 管理任务系统调度，包含调度算法
//! 
//! 2021年3月23日 zg


use crate::{interrupt::trap::Environment, libs::help::{switch_kernel, switch_user},
    memory::page_table::{SATP}, sync::mutex::Mutex};
use alloc::prelude::v1::*;
use super::{task_info::{ExecutionInfo, ProgramInfo, TaskState}};

pub enum ScheduleMethod{
    Rotation,
}

/// ## 任务管理器
/// 任务管理器假设任务存在两种状态：程序、执行。
/// 程序代表任务的整体，执行则是具体的执行、调度单位
/// 所以任务管理器应当支持针对程序及执行单位的对话操作
pub struct TaskManager<T1, T2> {
    scheduler : T1,
    task_pool : T2,
    mutex : Mutex,
}

/// ## 任务管理器基本功能实现
/// ID 分为任务 ID、程序 ID
/// 原则上只允许通过任务 ID 进行交互。而管理器对外则通过 Environment 或 hartid 交互
/// 基本的信息交流仅限于程序信息（ProgramInfo）、执行信息（ExecutionInfo）
impl<T1 : SchedulerOp, T2 : TaskPoolBasicOp> TaskManager<T1, T2> {
    pub const fn new(sche : T1, pool : T2)->Self {
        Self {
            scheduler : sche,
            task_pool : pool,
            mutex : Mutex::new(),
        }
    }

    pub fn start(&mut self, id : usize, hartid : usize) {
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.task_pool.set_task_exec(id, |info|{
            info.state = TaskState::Running;
            info.env.hartid = hartid;
        }).unwrap();
        self.switch_to(&info);
    }

    pub fn schedule(&mut self, env : &Environment) {
        let cur = self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == env.hartid
        });
        if let Some(cur) = cur {
            self.task_pool.set_task_exec(cur, |info|{
                info.state = TaskState::Waiting;
                info.env = env.clone();
            }).unwrap();
        }
        self.mutex.lock();
        let next = self.scheduler.schedule(&mut self.task_pool);
        if let Some(next) = next {
            let info = self.task_pool.get_task_exec(next).unwrap();
            self.task_pool.set_task_exec(next, |info|{
                info.state = TaskState::Running;
                info.env.hartid = env.hartid;
            }).unwrap();
            self.mutex.unlock();
            self.switch_to(&info);
        }
        self.mutex.unlock();
    }
    /// ### 创建任务并返还任务 ID
    pub fn create_task(&mut self, func : usize, is_kernel : bool)->Option<usize>{
        self.task_pool.create(func, is_kernel)
    }

    pub fn fork_task(&mut self, env : &Environment)->usize {
        self.task_pool.fork(env).unwrap()
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
    #[allow(dead_code)]
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

    pub fn task_exit(&mut self, hartid : usize) {
        self.task_pool.remove_task(hartid).unwrap();
    }

    pub fn program_exit(&mut self, hartid : usize) {
        let id = self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == hartid
        }).unwrap();
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.task_pool.remove_program(info.pid).unwrap();
        // let info = self.task_pool.get_task_prog(info.pid).unwrap();
        // self.task_pool.remove_task(info.pid).ok();
    }
    #[allow(dead_code)]
    pub fn get_task_exec_info(&mut self, id : usize)->Option<ExecutionInfo> {
        self.task_pool.get_task_exec(id)
    }
    #[allow(dead_code)]
    pub fn get_current_task(&mut self, env : &Environment)->usize {
        self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == env.hartid
        }).unwrap()
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

impl<T1 : SchedulerOp, T2 : TaskPoolBasicOp> TaskManager<T1, T2> {
    pub fn branch(&mut self, env:&Environment)->Option<usize> {
        self.task_pool.branch(env)
    }

    pub fn set_task_state(&mut self, id : usize, state : TaskState) {
        self.task_pool.set_task_exec(id, |info| {
            info.state = state;
        }).unwrap();
    }
}

/// ## 任务池操作要求
/// 与任务池的操作根据任务号进行，不获取引用，以便模块化
pub trait TaskPoolBasicOp {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize>;
    fn fork(&mut self, env : &Environment)->Option<usize>;
    fn branch(&mut self, env : &Environment)->Option<usize>;

    fn get_task_exec(&mut self, id : usize)->Option<ExecutionInfo>;
    fn get_task_prog(&mut self, id : usize)->Option<ProgramInfo>;

    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool;
    fn find<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool;

    fn operation_all<F>(&mut self,f:F) where F:FnMut(&ExecutionInfo);

    fn operation_once<F>(&mut self, f:F) where F:FnMut(&ExecutionInfo)->bool;

    fn set_task_exec<F>(&mut self, id:usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo);

    fn remove_task(&mut self, id : usize)->Result<(), ()>;
    fn remove_program(&mut self, id : usize)->Result<(), ()>;
}

/// ## 调度器操作要求
/// 算法实现由调度器自身决定
pub trait SchedulerOp{
    /// ### 调度器保存当前任务并选取下一个任务
    fn schedule<T:TaskPoolBasicOp>(&mut self, task_pool :&mut T)->Option<usize>;
    fn switch_method(&mut self, method : ScheduleMethod);
}

