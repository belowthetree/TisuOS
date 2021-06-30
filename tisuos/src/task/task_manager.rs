//! # 任务管理
//! 管理任务系统调度，包含调度算法
//! 严禁在中断期间使用任何内存管理功能以避免死锁！！！！
//! 
//! 2021年3月23日 zg


use crate::{filesystem::{pop_task_out, push_task_in, push_task_out}, interrupt::{environment::{Environment}, timer}, libs::{help::{start_kernel_process, switch_kernel_process, switch_user_process}, syscall::trigger_timer}, memory::ProgramArea};
use tisu_sync::SpinMutex;
use super::{require::{TaskPoolBasicOp, TaskPoolOp}, task_info::{ExecutionInfo, ProgramInfo, TaskState}};

pub enum ScheduleMethod{
    Rotation,
}

/// ## 任务管理器
/// 任务管理器假设任务存在两种状态：程序、执行。
/// 程序代表任务的整体，执行则是具体的执行、调度单位
/// 所以任务管理器应当支持针对程序及执行单位的对话操作
/// 同时任务管理器负责调用调度器及选择调度算法
pub struct TaskManager<T1, T2> {
    scheduler : T1,
    task_pool : T2,
    mutex : SpinMutex,
}

/// ## 任务管理器基本功能实现
/// ID 分为任务 ID、程序 ID
/// 原则上只允许通过任务 ID 进行交互。而管理器对外则通过 Environment 或 hartid 交互
/// 基本的信息交流仅限于程序信息（ProgramInfo）、执行信息（ExecutionInfo）
impl<T1 : SchedulerOp, T2 : TaskPoolOp> TaskManager<T1, T2> {
    pub const fn new(sche : T1, pool : T2)->Self {
        Self {
            scheduler : sche,
            task_pool : pool,
            mutex : SpinMutex::new(),
        }
    }

    /// 用于启动初始进程
    pub fn start(&mut self, id : usize, hartid : usize) {
        self.task_pool.set_task_exec(id, |info|{
            info.state = TaskState::Running;
            info.env.hartid = hartid;
        }).unwrap();
        let mut info = self.task_pool.get_task_exec(id).unwrap();
        unsafe {
            println!("before start");
            start_kernel_process(&mut info.env as *mut Environment as *mut u8);
        }
    }

    /// 调度器的调用及进程切换属于临界区，防止多核竞争错误
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
        if env.hartid == 0 {
            self.check_timer();
        }
        let next = self.scheduler.schedule(&mut self.task_pool);
        if let Some(next) = next {
            if self.task_pool.set_task_exec(next, |info|{
                info.state = TaskState::Running;
                info.env.hartid = env.hartid;
            }).is_ok() {
                if let Some(mut info) = self.task_pool.get_task_exec(next) {
                    self.mutex.unlock();
                    self.switch_to(&mut info);
                }
            }
        }
        self.mutex.unlock();
    }

    /// 本质是创建进程，返还主线程 ID
    pub fn create_task(&mut self, program : ProgramArea, env : &Environment)->Option<usize>{
        self.task_pool.create(program, env)
    }

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

    pub fn task_exit(&mut self, id : usize) {
        self.task_pool.remove_task(id).unwrap();
    }

    pub fn kill_task(&mut self, id : usize) {
        println!("remove task {}", id);
        let info = self.task_pool.get_task_exec(id).unwrap();
        if info.is_main {
            self.task_pool.remove_program(id).unwrap();
        }
        else {
            self.task_pool.remove_task(id).unwrap();
        }
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

    pub fn expand_stack(&mut self, id : usize)->Result<(), ()> {
        self.task_pool.expand_stack(id)
    }

    pub fn program_exit(&mut self, id : usize) {
        let info = self.task_pool.get_task_exec(id).unwrap();
        if info.is_main {
            self.task_pool.remove_program(id).unwrap();
        }
        else {
            self.task_pool.remove_task(id).unwrap();
        }
    }

    pub fn get_task_exec(&self, id : usize)->Option<ExecutionInfo> {
        self.task_pool.get_task_exec(id)
    }

    pub fn get_program_info(&mut self, pid : usize)->Option<ProgramInfo> {
        if let Some(tid) = self.task_pool.find(|info| {
            info.pid == pid
        }) {
            self.task_pool.get_task_prog(tid)
        }
        else {
            None
        }
    }

    pub fn find(&mut self, f : impl Fn(&ExecutionInfo)->bool)->Option<usize> {
        self.task_pool.find(f)
    }

    /// 调试用函数，待删除
    pub fn print(&self) {
        self.task_pool.print()
    }

    fn switch_to(&self, info : &mut ExecutionInfo) {
        trigger_timer();
        if info.is_kernel {
            unsafe {
                switch_kernel_process(&mut info.env as *mut Environment as *mut u8);
            }
        }
        else {
            unsafe {
                switch_user_process(&mut info.env as *mut Environment as *mut u8);
            }
        }
    }
}

/// 系统调用部分
impl<T1 : SchedulerOp, T2 : TaskPoolOp> TaskManager<T1, T2> {
    pub fn branch(&mut self, env:&Environment)->Option<usize> {
        self.task_pool.branch(env)
    }

    pub fn alloc_heap(&mut self, size: usize, id : usize)->(usize, usize) {
        self.task_pool.alloc_heap(size, id)
    }

    pub fn free_heap(&mut self, addr : usize, id : usize) {
        self.task_pool.free_heap(addr, id);
    }

    pub fn wait_task(&mut self, env: &Environment, target: usize) {
        let hartid = env.hartid;
        let id = self.task_pool.find(|info| {
            info.state == TaskState::Running && info.env.hartid == hartid
        }).unwrap();
        self.task_pool.wait_task(id, target);
        self.task_pool.set_task_exec(id, |info|{
            info.env = env.clone();
            info.env.epc += 4;
        }).unwrap();
    }

    pub fn sleep_timer(&mut self, env: &Environment, time : usize) {
        let hartid = env.hartid;
        let time = timer::get_micro_time() + time;
        let id = self.task_pool.find(|info|{
            info.state == TaskState::Running && info.env.hartid == hartid
        }).unwrap();
        self.task_pool.set_timer(id, time);
        self.task_pool.set_task_exec(id, |info| {
            info.env = env.clone();
            info.env.epc += 4;
        }).unwrap();
    }

    pub fn virt_to_phy(&self, id:usize, va:usize)->usize {
        self.task_pool.virt_to_phy(id, va)
    }

    pub fn sleep_task(&mut self, id : usize, env: &Environment)->Result<(), ()> {
        self.task_pool.set_task_exec(id, |info| {
            info.state = TaskState::Sleeping;
            info.env = env.clone();
            info.env.epc += 4;
        })
    }

    pub fn join(&mut self, id : usize, env: &Environment) {
        self.task_pool.join(id);
        self.task_pool.set_task_exec(id, |info| {
            info.env = env.clone();
            info.env.epc += 4;
        }).unwrap();
    }

    /// 检查是否有任务到了该唤醒的时间
    fn check_timer(&mut self) {
        let time = timer::get_micro_time();
        self.task_pool.check_timer(time);
    }
}

/// 资源部分
impl<T1 : SchedulerOp, T2 : TaskPoolOp> TaskManager<T1, T2> {
    pub fn push_file(&mut self, task_id : usize, file_id:usize) {
        self.task_pool.push_file(task_id, file_id);
    }

    pub fn release_file(&mut self, task_id : usize, file_id:usize) {
        self.task_pool.release_file(task_id, file_id);
    }

    pub fn stdout(&mut self, id:usize, data:&[u8]) {
        // println!("mgr stdout");
        for c in data {
            push_task_out(id, *c as char);
        }
        // println!("after mgr stdout");
    }

    #[allow(dead_code)]
    pub fn stdin(&mut self, id:usize, data:&[u8]) {
        for c in data {
            push_task_in(id, *c as char);
        }
    }

    pub fn get_stdout(&mut self, id:usize, data:&mut [u8])->usize {
        let mut idx = 0;
        while let Some(c) = pop_task_out(id) {
            if idx >= data.len() {
                break;
            }
            data[idx] = c as u8;
            idx += 1;
        }
        idx
    }
}

/// ## 调度器操作要求
/// 算法实现由调度器自身决定
pub trait SchedulerOp{
    /// ### 调度器保存当前任务并选取下一个任务
    fn schedule<T:TaskPoolBasicOp>(&mut self, task_pool :&mut T)->Option<usize>;
    fn switch_method(&mut self, method : ScheduleMethod);
}

