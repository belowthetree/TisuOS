//! # 线程（有空再做）
//! 线程是进程的缩减版，但是不能独立存在，必须依赖于进程
//! 线程只保留自身的寄存器、栈环境
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]
#[derive(PartialEq, Debug)]
pub enum ThreadState{
    Running,
    Waiting,
    Sleeping,
}

/// ## 线程
/// 不保存堆信息，pid 记录所属进程号
pub struct Thread{
    pub env : Environment,
    pub state : ThreadState,
    pub stack_top : *mut u8,
    pub pid : usize,
    pub tid : usize,
    pub hartid : usize,
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
        let pt = satp.get_page_table();
        unsafe {
            if p.is_kernel{
                for i in 0..STACK_PAGE_NUM{
                    let addr = stack_bottom as usize + i * PAGE_SIZE;
                    (*pt).map_kernel_data(addr, addr);
                }
            }
            else {
                for i in 0..STACK_PAGE_NUM{
                    let addr = stack_bottom as usize + i * PAGE_SIZE;
                    (*pt).map_user_data(addr, addr);
                }
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
            hartid : 0,
            is_kernel : p.is_kernel,
        })
    }

}

impl Drop for Thread{
    fn drop(&mut self) {
        let stack_bottom = self.stack_top as usize - PAGE_SIZE * STACK_PAGE_NUM;
        free_page(stack_bottom as *mut u8);
        drop_thread(self.pid, self.tid);
    }
}

extern "C" {
    fn switch_user_process(env : *mut u8) -> usize;
    fn switch_kernel_process(env : *mut u8) -> usize;
    fn thread_exit();
}

static mut THREAD_CNT : usize = 1;
static mut THREAD_LOCK : Mutex = Mutex::new();
static mut THREAD_LIST : Option<Vec<Thread>> = None;
static mut ENVIRONMENT : *mut Environment = null_mut();

pub fn init(){
    unsafe {
        THREAD_LIST = Some(Vec::<Thread>::new());
        ENVIRONMENT = alloc_kernel(size_of::<Environment>()) as *mut Environment;
    }
}

pub fn create_thread<'a>(func : usize, p : &Process)->Option<&'a mut Thread>{
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            if let Some(t) = Thread::new(func, p){
                list.push(t);
                let rt = list.last_mut().unwrap();
                THREAD_LOCK.unlock();
                return Some(rt);
            }
        }
        THREAD_LOCK.unlock();
        None
    }
}

pub fn wake_up(tid : usize){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(threads) = &mut THREAD_LIST{
            for thread in threads {
                if thread.tid == tid{
                    thread.state = ThreadState::Waiting;
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn schedule(env : &Environment){
    save_current(env);
    switch_next();
}

pub fn delete_current_thread(){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            let hartid = cpu::get_hartid();
            for (idx, thread) in list.iter().enumerate() {
                if thread.state == ThreadState::Running && thread.hartid == hartid{
                    process::drop_thread(thread.pid, thread.tid);
                    list.remove(idx);
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn run(tid : usize){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            for t in list{
                if t.tid == tid{
                    t.state = ThreadState::Running;
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn get_thread_by_id<'a>(tid : usize)->Option<&'a mut Thread>{
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            for t in list{
                if t.tid == tid{
                    THREAD_LOCK.unlock();
                    return Some(t);
                }
            }
        }
        THREAD_LOCK.unlock();
        None
    }
}

pub fn delete(tid : usize){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            for (idx, t) in list.iter().enumerate() {
                if t.tid == tid{
                    list.remove(idx);
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

fn save_current(env : &Environment){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(threads) = &mut THREAD_LIST{
            let hartid = cpu::get_hartid();
            for t in threads{
                if t.state == ThreadState::Running && t.hartid == hartid {
                    t.env.copy(env);
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

fn switch_next(){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(threads) = &mut THREAD_LIST{
            let hartid = cpu::get_hartid();
            for (idx, t) in threads.iter_mut().enumerate(){
                if t.state == ThreadState::Running && t.hartid == hartid {
                    t.state = ThreadState::Waiting;
                    threads.rotate_left(idx + 1);
                    break;
                }
            }
            for t in threads{
                if t.state == ThreadState::Waiting {
                    t.state = ThreadState::Running;
                    t.hartid = hartid;
                    (*ENVIRONMENT).copy(&t.env);
                    let is_kernel = t.is_kernel;
                    THREAD_LOCK.unlock();
                    if is_kernel{
                        switch_kernel_process(ENVIRONMENT as *mut u8);
                    }
                    else{
                        switch_user_process(ENVIRONMENT as *mut u8);
                    }
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn get_current_thread<'a>()->Option<&'a Thread>{
    unsafe {
        THREAD_LOCK.lock();
        if let Some(threads) = &mut THREAD_LIST{
            let hartid = cpu::get_hartid();
            for t in threads{
                if t.state == ThreadState::Running && t.hartid == hartid{
                    THREAD_LOCK.unlock();
                    return Some(t);
                }
            }
        }
        THREAD_LOCK.unlock();
        None
    }
}

pub fn print(){
    unsafe {
        if let Some(list) = &THREAD_LIST{
            for t in list{
                println!("thread {}, state {:?} in process {}", t.tid, t.state, t.pid);
            }
        }
        else{
            println!("no");
        }
    }
}

use core::{mem::size_of, ptr::null_mut};

use page::{PAGE_SIZE, free_page};
use alloc::{prelude::v1::*};

use crate::{cpu, uart, interrupt::trap::{Environment, Register}, memory::{global_allocator::alloc_kernel, page, page_table::{SATP}}, sync::Mutex};

use super::process::{self, Process, STACK_PAGE_NUM, drop_thread};



