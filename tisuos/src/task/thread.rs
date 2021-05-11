//! # 线程（有空再做）
//! 线程是进程的缩减版，但是不能独立存在，必须依赖于进程
//! 线程只保留自身的寄存器、栈环境
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

extern "C" {
    fn thread_exit();
    fn process_exit();
}

/// ## 线程
/// 不保存堆信息，pid 记录所属进程号
pub struct Thread{
    pub info : ExecutionInfo,
}

/// ## 线程功能
/// fork 拷贝原线程栈内容然后创建新的分支
/// branch 直接根据传入地址（默认是原线程的第二个参数）创建新的线程
impl Thread {
    /// ## 为进程新建主线程线程
    /// 为线程映射栈，同时分别为用户、内核进程映射不同的内存区域
    pub fn new(p : &Process)->Option<Self>{
        let mut env = Environment::new();
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT.add() };
        let stack_top;
        stack_bottom = Thread::stack(p.is_kernel).expect("thread new alloc stack fail");
        Thread::map_stack(&p.info.satp, stack_bottom, p.is_kernel);

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = p.entry();
        env.satp = p.info.satp.val();
        env.regs[Register::A0.val()] = tid;
        env.regs[Register::SP.val()] = stack_top;
        env.regs[Register::RA.val()] = process_exit as usize;
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
            }
        })
    }

    /// ## 原地分支执行
    /// 将会拷贝原线程栈环境
    pub fn fork(src_th : &Thread)->Option<Self>{
        let mut env = src_th.info.env;
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT.add() };
        let stack_top;
        if let Some(st) = Thread::stack(src_th.info.is_kernel) {
            stack_bottom = st;
        }
        else {
            return None;
        }
        Thread::map_stack(
            &SATP::from(src_th.info.env.satp), stack_bottom, src_th.info.is_kernel);
        unsafe {
            let stack_size = STACK_PAGE_NUM * PAGE_SIZE;
            let src = (src_th.info.stack_top as usize - stack_size) as *mut u8;
            stack_bottom.copy_from(src, stack_size);
        }

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = src_th.info.env.epc + 4;
        env.regs[Register::SP.val()] =
            stack_top - (src_th.info.stack_top as usize - src_th.info.env.regs[Register::SP.val()]);
        // println!("thread fork src tid {} stack {:x} sp {:x}, new stack {:x} sp {:x} tid {}",
        // src_th.tid, src_th.stack_top as usize, src_th.env.regs[Register::SP.val()], stack_top,
        //     env.regs[Register::SP.val()], tid);
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
            }
        })
    }

    /// ## 函数分支执行
    /// 区别于 fork，从传入的地址（应该是一个函数地址）开始执行，用全新的栈环境
    pub fn branch(src_th : &Thread)->Option<Self>{
        let mut env = src_th.info.env;
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT.add() };
        let stack_top;
        stack_bottom = Thread::stack(src_th.info.is_kernel).expect("branch alloc stack fail");
        Thread::map_stack(
            &SATP::from(src_th.info.env.satp), stack_bottom, src_th.info.is_kernel);

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = src_th.info.env.regs[Register::A1.val()];
        // println!("thread branch src tid {} stack {:x} sp {:x}, new stack {:x} sp {:x} tid {}
        //     epc {:x}",
        // src_th.tid, src_th.stack_top as usize, src_th.env.regs[Register::SP.val()], stack_top,
        //     env.regs[Register::SP.val()], tid, env.epc);
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
            }
        })
    }

    pub fn sleep(&mut self) {
        self.info.state = TaskState::Sleeping;
    }

    pub fn wake(&mut self) {
        self.info.state = TaskState::Waiting
    }

    fn stack(is_kernel : bool)->Option<*mut u8> {
        if is_kernel {
            get_manager().kernel_page(STACK_PAGE_NUM)
        }
        else {
            get_manager().user_page(STACK_PAGE_NUM)
        }
    }

    fn map_stack(satp : &SATP, stack_bottom : *mut u8, is_kernel : bool) {
        if is_kernel {
            return;
        }
        for i in 0..STACK_PAGE_NUM{
            let addr = stack_bottom as usize + i * PAGE_SIZE;
            satp.map_data(addr, addr, is_kernel);
        }
    }
}

/// ## 信息相关操作
impl Thread {
    pub fn get_exec_info(&self)->ExecutionInfo {
        self.info
    }

    pub fn set_exec_info(&mut self, info : &ExecutionInfo) {
        self.info.state = info.state;
        self.info.stack_top = info.stack_top;
        self.info.pid = info.pid;
        self.info.tid = info.tid;
        self.info.env = info.env;
    }

    pub fn save(&mut self, env : &Environment) {
        self.info.env = env.clone();
    }
}

impl Drop for Thread{
    fn drop(&mut self) {
        let stack_bottom = self.info.stack_top as usize - PAGE_SIZE * STACK_PAGE_NUM;
        get_manager().free_page(stack_bottom as *mut u8);
    }
}

static mut THREAD_CNT : AtomCounter = AtomCounter::new();

/// ## 初始化临时环境内存
pub fn init(){
}


use tisu_memory::MemoryOp;
use tisu_sync::AtomCounter;

use crate::{
    interrupt::environment::{Environment, Register},
    memory::{config::PAGE_SIZE, get_manager, map::SATP}
};

use super::{
    task_info::ExecutionInfo, process::{Process, STACK_PAGE_NUM},
    task_info::TaskState
};



