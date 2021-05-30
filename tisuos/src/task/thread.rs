//! # 线程（有空再做）
//! 线程是进程的缩减版，但是不能独立存在，必须依赖于进程
//! 线程只保留自身的寄存器、栈环境
//! 
//! 2021年1月1日 zg


extern "C" {
    fn thread_exit();
    fn process_exit();
}

pub const MAX_STACK_PAGE : usize = 64;
pub const STACK_PAGE_NUM : usize = 16;

/// ## 线程
/// 不保存堆信息，pid 记录所属进程号
pub struct Thread{
    pub info : ExecutionInfo,
    stack : TaskStack,
}

/// ## 线程功能
/// fork 拷贝原线程栈内容然后创建新的分支
/// branch 直接根据传入地址（默认是原线程的第二个参数）创建新的线程
impl Thread {
    /// ## 为进程新建主线程线程
    /// 为线程映射栈，同时分别为用户、内核进程映射不同的内存区域
    pub fn new(p : &Process, src_env : &Environment)->Option<Self>{
        let mut env = Environment::new();
        let tid = unsafe{ THREAD_CNT.add() };
        let t = unsafe {MEMORY_END} / PAGE_SIZE * PAGE_SIZE + MAX_HEAP_SIZE;
        let stack_top = t + (tid + 1) * MAX_STACK_PAGE * 2 * PAGE_SIZE;
        env.epc = p.entry();
        env.satp = p.info.satp.val();
        env.regs[Register::A0.val()] = src_env.a3();
        env.regs[Register::A1.val()] = src_env.a4();
        env.regs[Register::A2.val()] = src_env.a5();
        env.regs[Register::SP.val()] = stack_top;
        env.regs[Register::RA.val()] = process_exit as usize;
        let mut stack = TaskStack::new(tid, stack_top, MAX_STACK_PAGE, p.is_kernel);
        stack.expand(STACK_PAGE_NUM, &p.info.satp).unwrap();
        Some(Self{
            info : ExecutionInfo {
                priority : 1,
                env,
                state : TaskState::Sleeping,
                stack_top : stack_top as *mut u8,
                pid : p.info.pid,
                tid,
                is_kernel : p.is_kernel,
                is_main : true,
                trigger_time : 0,
            },
            stack,
        })
    }

    /// ## 原地分支执行
    /// 将会拷贝原线程栈环境
    pub fn fork(src_th : &Thread)->Option<Self>{
        let mut env = src_th.info.env;
        let tid = unsafe{ THREAD_CNT.add() };
        let t = unsafe {MEMORY_END} / PAGE_SIZE * PAGE_SIZE + MAX_HEAP_SIZE;
        let stack_top = t + (tid + 1) * MAX_STACK_PAGE * 2 * PAGE_SIZE;
        let mut stack = TaskStack::new(tid, stack_top, MAX_STACK_PAGE, src_th.info.is_kernel);
        stack.copy(&src_th.stack, &SATP::from(src_th.info.env.satp));

        env.epc = src_th.info.env.epc + 4;
        env.regs[Register::SP.val()] = stack_top -
            (src_th.info.stack_top as usize - src_th.info.env.regs[Register::SP.val()]);
        env.regs[Register::A0.val()] = 0;
        Some(Self{
            info : ExecutionInfo{
                priority : 1,
                env,
                state : TaskState::Waiting,
                stack_top : stack_top as *mut u8,
                pid : src_th.info.pid,
                tid,
                is_kernel : src_th.info.is_kernel,
                is_main : false,
                trigger_time : 0,
            },
            stack,
        })
    }

    /// ## 函数分支执行
    /// 区别于 fork，从传入的地址（应该是一个函数地址）开始执行，用全新的栈环境
    pub fn branch(src_th : &Thread)->Option<Self>{
        let mut env = src_th.info.env;
        let tid = unsafe{ THREAD_CNT.add() };
        let t = unsafe {MEMORY_END} / PAGE_SIZE * PAGE_SIZE + MAX_HEAP_SIZE;
        let stack_top = t + (tid + 1) * MAX_STACK_PAGE * 2 * PAGE_SIZE;
        env.epc = src_th.info.env.regs[Register::A1.val()];
        env.regs[Register::A0.val()] = env.a2();
        env.regs[Register::RA.val()] = thread_exit as usize;
        env.regs[Register::SP.val()] = stack_top;
        Some(Self{
            info : ExecutionInfo {
                priority : 1,
                env,
                state : TaskState::Waiting,
                stack_top : stack_top as *mut u8,
                pid : src_th.info.pid,
                tid,
                is_kernel : src_th.info.is_kernel,
                is_main : false,
                trigger_time : 0,
            },
            stack : TaskStack::new(tid, stack_top, MAX_STACK_PAGE, src_th.info.is_kernel),
        })
    }

    pub fn sleep(&mut self) {
        self.info.state = TaskState::Sleeping;
    }

    pub fn wake(&mut self) {
        self.info.state = TaskState::Waiting
    }

    pub fn expand_stack(&mut self, satp : &SATP)->Result<(), ()> {
        self.stack.expand(STACK_PAGE_NUM, satp)
    }
}

/// ## 信息相关操作
impl Thread {
    pub fn get_exec_info(&self)->ExecutionInfo {
        self.info
    }

    pub fn save(&mut self, env : &Environment) {
        self.info.env = env.clone();
    }

    pub fn virt_to_phy(&self, va:usize)->usize {
        self.stack.virt_to_phy(va)
    }
}


static mut THREAD_CNT : AtomCounter = AtomCounter::new();

/// ## 初始化临时环境内存
pub fn init(){
}


use tisu_sync::AtomCounter;

use crate::{interrupt::environment::{Environment, Register}, memory::{TaskStack, config::{MEMORY_END, PAGE_SIZE}, map::SATP}, task::process::MAX_HEAP_SIZE};

use super::{
    task_info::ExecutionInfo, process::Process,
    task_info::TaskState
};



