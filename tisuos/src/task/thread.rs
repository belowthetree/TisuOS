//! # 线程（有空再做）
//! 线程是进程的缩减版，但是不能独立存在，必须依赖于进程
//! 线程只保留自身的寄存器、栈环境
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

extern "C" {
    pub fn switch_user_process(env : *mut u8) -> usize;
    pub fn switch_kernel_process(env : *mut u8) -> usize;
    fn thread_exit();
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ThreadState{
    Running,
    Waiting,
    Sleeping,
}

impl ThreadState {
    pub fn to_task_state(&self)->TaskState {
        match self {
            ThreadState::Running => {TaskState::Running}
            ThreadState::Waiting => {TaskState::Waiting}
            ThreadState::Sleeping => {TaskState::Sleeping}
        }
    }
}

/// ## 线程
/// 不保存堆信息，pid 记录所属进程号
pub struct Thread{
    pub env : Environment,
    pub state : ThreadState,
    pub stack_top : *mut u8,
    pub pid : usize,
    pub tid : usize,
    pub is_kernel : bool,
}

impl Thread {
    pub fn new(func : usize, p : &Process)->Option<Self>{
        let mut env = Environment::new();
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT };
        let stack_top;
        if p.is_kernel{
            stack_bottom = page::alloc_kernel_page(STACK_PAGE_NUM);
        }
        else{
            stack_bottom = page::alloc_user_page(STACK_PAGE_NUM);
        }
        if stack_bottom.is_null(){
            return None;
        }
        let satp = SATP::from(p.satp);
        let pt = satp.get_page_table().unwrap();
        if p.is_kernel{
            for i in 0..STACK_PAGE_NUM{
                let addr = stack_bottom as usize + i * PAGE_SIZE;
                pt.map_kernel_data(addr, addr);
            }
        }
        else {
            for i in 0..STACK_PAGE_NUM{
                let addr = stack_bottom as usize + i * PAGE_SIZE;
                pt.map_user_data(addr, addr);
            }
        }
        

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = func;
        env.satp = p.satp;
        env.regs[Register::SP.val()] = stack_top;
        env.regs[Register::RA.val()] = thread_exit as usize;
        unsafe {
            THREAD_CNT = THREAD_CNT + 1;
            if THREAD_CNT == 0{
                THREAD_CNT += 1;
            }
        }
        Some(Self{
            env : env,
            state : ThreadState::Waiting,
            stack_top : stack_top as *mut u8,
            pid : p.pid,
            tid : tid,
            is_kernel : p.is_kernel,
        })
    }
    pub fn copy(info : &ExecutionInfo)->Option<Self>{
        let mut env = info.env;
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT };
        let stack_top;
        if info.is_kernel{
            stack_bottom = page::alloc_kernel_page(STACK_PAGE_NUM);
        }
        else{
            stack_bottom = page::alloc_user_page(STACK_PAGE_NUM);
        }
        if stack_bottom.is_null(){
            return None;
        }
        let satp = SATP::from(info.env.satp);
        let pt = satp.get_page_table().unwrap();
        unsafe {
            if info.is_kernel{
                for i in 0..STACK_PAGE_NUM{
                    let addr = stack_bottom as usize + i * PAGE_SIZE;
                    pt.map_kernel_data(addr, addr);
                }
            }
            else {
                for i in 0..STACK_PAGE_NUM{
                    let addr = stack_bottom as usize + i * PAGE_SIZE;
                    pt.map_user_data(addr, addr);
                }
            }
            let stack_size = STACK_PAGE_NUM * PAGE_SIZE;
            let src = (info.stack_top as usize - stack_size) as *mut u8;
            stack_bottom.copy_from(src, stack_size);
        }

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = info.env.epc + 4;
        env.regs[Register::SP.val()] = stack_top - (info.stack_top as usize - info.env.regs[Register::SP.val()]);
        println!("thread copy src tid {} stack {:x} sp {:x}, new stack {:x} sp {:x} tid {}", info.tid,
            info.stack_top as usize, info.env.regs[Register::SP.val()], stack_top,
            env.regs[Register::SP.val()], tid);
        env.regs[Register::A0.val()] = tid;
        unsafe {
            THREAD_CNT = THREAD_CNT + 1;
            if THREAD_CNT == 0{
                THREAD_CNT += 1;
            }
        }
        Some(Self{
            env : env,
            state : ThreadState::Waiting,
            stack_top : stack_top as *mut u8,
            pid : info.pid,
            tid : tid,
            is_kernel : info.is_kernel,
        })
    }
    pub fn get_exec_info(&self)->ExecutionInfo {
        ExecutionInfo::from_thread(self)
    }
    pub fn set_exec_info(&mut self, info : &ExecutionInfo) {
        self.env = info.env;
    }

}

impl Drop for Thread{
    fn drop(&mut self) {
        let stack_bottom = self.stack_top as usize - PAGE_SIZE * STACK_PAGE_NUM;
        free_page(stack_bottom as *mut u8);
        delete_pipe(self.tid);
    }
}

static mut THREAD_CNT : usize = 1;
static mut THREAD_LOCK : Mutex = Mutex::new();
static mut THREAD_LIST : Option<Vec<Thread>> = None;
static mut ENVIRONMENT : [*mut Environment;4] = [null_mut();4];
static mut RECORD : [usize;4] = [0;4];

/// ## 初始化临时环境内存
pub fn init(){
    unsafe {
        THREAD_LIST = Some(Vec::<Thread>::new());
        for i in 0..4{
            ENVIRONMENT[i] = alloc(size_of::<Environment>(), true).unwrap() as *mut Environment;
        }
    }
}

use core::{mem::size_of, ptr::null_mut};

use page::{PAGE_SIZE, free_page};
use alloc::{prelude::v1::*};

use crate::{uart, interrupt::trap::{Environment, Register}, memory::{allocator::alloc, page, page_table::{SATP}}, sync::Mutex};

use super::{delete_pipe, info::{ExecutionInfo}, process::{Process, STACK_PAGE_NUM}, task_manager::TaskState};



