//! # 任务管理
//! 管理任务系统调度，包含调度算法
//! 
//! 2021年3月23日 zg


use crate::{
    interrupt::trap::Environment,
    libs::help::{switch_kernel, switch_user},
    memory::block::Block,
};
use tisu_sync::SpinMutex;
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
    mutex : SpinMutex,
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
            mutex : SpinMutex::new(),
        }
    }

    pub fn start(&mut self, id : usize, hartid : usize) {
        self.task_pool.set_task_exec(id, |info|{
            info.state = TaskState::Running;
            info.env.hartid = hartid;
        }).unwrap();
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.switch_to(&info);
    }

    pub fn schedule(&mut self, env : &Environment) {
        let cur = self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == env.hartid
        });
        if let Some(cur) = cur {
            self.task_pool.set_task_exec(cur, |info|{
                if info.pid == 1 {
                    println!("pid 1 save");
                }
                info.state = TaskState::Waiting;
                info.env = env.clone();
            }).unwrap();
        }
        self.mutex.lock();
        let next = self.scheduler.schedule(&mut self.task_pool);
        if let Some(next) = next {
            self.task_pool.set_task_exec(next, |info|{
                info.state = TaskState::Running;
                info.env.hartid = env.hartid;
            }).unwrap();
            let info = self.task_pool.get_task_exec(next).unwrap();
            self.mutex.unlock();
            self.switch_to(&info);
        }
        self.mutex.unlock();
    }
    /// ### 创建任务并返还任务 ID
    pub fn create_task(&mut self, func : usize, is_kernel : bool)->Option<usize>{
        self.task_pool.create(func, is_kernel)
    }
    #[allow(dead_code)]
    pub fn wake_task(&mut self, id : usize) {
        self.task_pool.set_task_exec(id, |info| {
            if info.tid == id {
                info.state = TaskState::Waiting
            }
        }).unwrap();
    }

    pub fn fork_task(&mut self, env : &Environment)->usize {
        self.task_pool.fork(env).unwrap()
    }
    #[allow(dead_code)]
    pub fn map_code(&mut self, id : usize, va : usize, pa : usize) {
        let info = self.task_pool.get_task_prog(id).unwrap();
        info.satp.map_code(va, pa, info.is_kernel)
    }
    #[allow(dead_code)]
    pub fn map_data(&mut self, id : usize, va : usize, pa : usize) {
        let info = self.task_pool.get_task_prog(id).unwrap();
        info.satp.map_data(va, pa, info.is_kernel);
    }

    pub fn task_exit(&mut self, hartid : usize) {
        self.task_pool.remove_task(hartid).unwrap();
    }
    #[allow(dead_code)]
    pub fn remove_program(&mut self, id : usize) {
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.task_pool.remove_program(info.pid).unwrap();
    }

    pub fn get_current_task(&mut self, hartid : usize)->Option<(ExecutionInfo, ProgramInfo)> {
        if let Some(id) = self.task_pool.find(|info| {
            info.state == TaskState::Running && info.env.hartid == hartid
        }) {
            let info1 = self.task_pool.get_task_exec(id).unwrap();
            let info2 = self.task_pool.get_task_prog(id).unwrap();
            Some((info1, info2))
        }
        else {
            None
        }
    }

    pub fn program_exit(&mut self, hartid : usize) {
        let id = self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == hartid
        }).unwrap();
        let info = self.task_pool.get_task_exec(id).unwrap();
        self.task_pool.remove_program(info.pid).unwrap();
        println!("after remove pid {}", info.pid);
    }
    #[allow(dead_code)]
    pub fn get_program_info(&self, id : usize)->ProgramInfo {
        self.task_pool.get_task_prog(id).unwrap()
    }
    #[allow(dead_code)]
    pub fn get_exec_info(&self, id : usize)->ExecutionInfo {
        self.task_pool.get_task_exec(id).unwrap()
    }

    pub fn print(&self) {
        self.task_pool.print()
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

    fn get_task_exec(&self, id : usize)->Option<ExecutionInfo>;
    fn get_task_prog(&self, id : usize)->Option<ProgramInfo>;

    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool;
    fn find<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool;

    fn operation_all<F>(&mut self,f:F) where F:FnMut(&ExecutionInfo);

    fn operation_once<F>(&mut self, f:F) where F:FnMut(&ExecutionInfo)->bool;

    fn set_task_exec<F>(&mut self, id:usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo);

    fn send_task_msg(&mut self, id : usize, msg : &Block<u8>);

    fn set_task_prog<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut ProgramInfo);

    fn remove_task(&mut self, id : usize)->Result<(), ()>;
    fn remove_program(&mut self, id : usize)->Result<(), ()>;

    fn print(&self);
}

/// ## 调度器操作要求
/// 算法实现由调度器自身决定
pub trait SchedulerOp{
    /// ### 调度器保存当前任务并选取下一个任务
    fn schedule<T:TaskPoolBasicOp>(&mut self, task_pool :&mut T)->Option<usize>;
    fn switch_method(&mut self, method : ScheduleMethod);
}

