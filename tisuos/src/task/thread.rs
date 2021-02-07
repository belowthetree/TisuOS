//! # 线程（有空再做）
//! 线程是进程的缩减版，但是不能独立存在，必须依赖于进程
//! 线程只保留自身的寄存器、栈环境
//! 
//! 2021年1月1日 zg

#![allow(dead_code)]

extern "C" {
    fn switch_user_process(env : *mut u8) -> usize;
    fn switch_kernel_process(env : *mut u8) -> usize;
    fn thread_exit();
}

#[derive(PartialEq, Debug, Clone, Copy)]
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
            is_kernel : p.is_kernel,
        })
    }
    pub fn copy(src_env : &Environment)->Option<Self>{
        let thread = get_current_thread(src_env.hartid).unwrap();
        let mut env = src_env.clone();
        let stack_bottom;
        let tid = unsafe{ THREAD_CNT };
        let stack_top;
        if thread.is_kernel{
            stack_bottom = page::alloc_kernel_page(STACK_PAGE_NUM);
        }
        else{
            stack_bottom = page::alloc_user_page(STACK_PAGE_NUM);
        }
        if stack_bottom.is_null(){
            return None;
        }
        let satp = SATP::from(thread.env.satp);
        let pt = satp.get_page_table();
        unsafe {
            if thread.is_kernel{
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
            let stack_size = STACK_PAGE_NUM * PAGE_SIZE;
            let src = (thread.stack_top as usize - stack_size) as *mut u8;
            stack_bottom.copy_from(src, stack_size);
        }

        stack_top = stack_bottom as usize + PAGE_SIZE * STACK_PAGE_NUM;
        env.epc = src_env.epc + 4;
        env.regs[Register::SP.val()] = stack_top - (thread.stack_top as usize - src_env.regs[Register::SP.val()]);
        println!("thread copy src tid {} stack {:x} sp {:x}, new stack {:x} sp {:x} tid {}", thread.tid,
            thread.stack_top as usize, src_env.regs[Register::SP.val()], stack_top,
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
            pid : thread.pid,
            tid : tid,
            is_kernel : thread.is_kernel,
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

pub fn list() {
    unsafe {
        for tid in RECORD.iter() {
            print!("{} ", tid);
        }
        println!();
    }
}

/// ## 创建并返回线程的引用
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

/// ## 根据已有的线程创建分支
/// 执行地址根据传入的环境而定，用于系统调用
pub fn fork(env : &Environment){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            if let Some(t) = Thread::copy(env){
                list.push(t);
            }
        }
        THREAD_LOCK.unlock();
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

/// ## 调度函数
pub fn schedule(env : &Environment){
    save_current(env);
    switch_next(env.hartid);
}

pub fn delete_current_thread(hartid : usize){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            for (idx, thread) in list.iter().enumerate() {
                if thread.state == ThreadState::Running && thread.env.hartid == hartid{
                    process::drop_thread(thread.pid, thread.tid);
                    list.remove(idx);
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn run(tid : usize, hartid : usize){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(list) = &mut THREAD_LIST{
            for t in list{
                if t.tid == tid{
                    t.state = ThreadState::Running;
                    t.env.hartid = hartid;
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

pub fn save_current(env : &Environment){
    unsafe {
        THREAD_LOCK.lock();
        if let Some(threads) = &mut THREAD_LIST{
            let hartid = env.hartid;
            for t in threads{
                if t.state == ThreadState::Running && t.env.hartid == hartid {
                    t.env.copy(env);
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
    }
}

pub fn switch_next(hartid : usize){
    unsafe {
        THREAD_LOCK.lock();
        let mut is_kernel = false;
        let mut sche = false;
        if let Some(threads) = &mut THREAD_LIST{
            for (idx, t) in threads.iter_mut().enumerate(){
                if t.state == ThreadState::Running && t.env.hartid == hartid {
                    t.state = ThreadState::Waiting;
                    threads.rotate_left(idx + 1);
                    break;
                }
            }
            for t in threads.iter_mut(){
                if t.state == ThreadState::Waiting {
                    t.state = ThreadState::Running;
                    t.env.hartid = hartid;
                    // println!("thread {} run in hartid {}", t.tid, hartid);
                    (*ENVIRONMENT[hartid]).copy(&t.env);
                    is_kernel = t.is_kernel;
                    sche = true;
                    RECORD[hartid] = t.tid;
                    break;
                }
            }
        }
        THREAD_LOCK.unlock();
        if !sche {
            return;
        }
        if is_kernel{
            switch_kernel_process(ENVIRONMENT[hartid] as *mut u8);
        }
        else{
            switch_user_process(ENVIRONMENT[hartid] as *mut u8);
        }
    }
}

fn get_current_thread(hartid : usize)->Option<Info>{
    unsafe {
        if let Some(threads) = &mut THREAD_LIST{
            for t in threads{
                println!("{} thread {}, state {:?}, hartid {}", hartid, t.tid, t.state, t.env.hartid);
                if t.state == ThreadState::Running && t.env.hartid == hartid{
                    return Some(Info::from_thread(t));
                }
            }
        }
        None
    }
}

pub fn get_current_thread_pid(hartid : usize)->Option<usize> {
    unsafe {
        THREAD_LOCK.lock();
        let mut rt = None;
        if let Some(threads) = &mut THREAD_LIST{
            for t in threads{
                println!("{} thread {}, state {:?}, hartid {}", hartid, t.tid, t.state, t.env.hartid);
                if t.state == ThreadState::Running && t.env.hartid == hartid{
                    rt = Some(t.pid);
                }
            }
        }
        THREAD_LOCK.unlock();
        rt
    }
}

pub fn print(){
    unsafe {
        if let Some(list) = &THREAD_LIST{
            for t in list{
                println!("thread {}, state {:?} in process {}, hartid {}", t.tid, t.state, t.pid, t.env.hartid);
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

use crate::{uart, interrupt::trap::{Environment, Register}, memory::{allocator::alloc, page, page_table::{SATP}}, sync::Mutex};

use super::{info::Info, process::{self, Process, STACK_PAGE_NUM, drop_thread}};



