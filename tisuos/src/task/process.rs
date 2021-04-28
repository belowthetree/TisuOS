//! # 进程
//! 进程作为程序标志，为程序保存基本信息
//!
//! 2020年12月12日 zg

use alloc::{prelude::v1::*};


pub static mut PID_CNT : AtomCounter = AtomCounter::new();
pub static STACK_PAGE_NUM : usize = 16;
#[allow(dead_code)]
pub static HEAP_SIZE : usize = 4096;

/// ## ProcessState
/// 进程目前分为三种状态：Waiting、Sleeping、Running
/// 每个核心只有一个 Running 的进程，只有 Waiting 的进程能够被调度
/// Sleeping 的进程除非被唤醒，否则永远沉睡
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum ProcessState{
    Scheduling = 1,
    Sleeping = 3,
}

impl PartialEq for ProcessState{
    fn eq(&self, other: &Self) -> bool {
        (*self as usize) == (*other as usize)
    }
}

/// ## 进程信息结构体
/// 保存环境、栈堆信息（预计增加唤醒时间、优先级）
/// pages 属于进程占用的物理页
pub struct Process{
    pub info : ProgramInfo,
    pub tid : Vec<usize>,
    pub is_kernel : bool,
}

impl Process {
    /// ## 新建一个进程
    /// 进程仅进行基本信息的初始化：堆链表、satp
    pub fn new(is_kernel : bool) -> Option<Self> {
        let pid = unsafe {PID_CNT.add()};
        let info = ProgramInfo{
            pid,
            satp : SATP::new(),
            state: TaskState::Running,
            is_kernel,
        };
        let rt = Process{
            info,
            is_kernel,
            tid : Vec::<usize>::new(),
        };
        Some(rt)
    }

    pub fn get_prog_info(&self)->ProgramInfo {
        self.info
    }

    pub fn set_prog_info(&mut self, info : ProgramInfo) {
        self.info = info;
    }
}
/// 进程的释放发生在被从调度队列中剔除
impl Drop for Process{
    fn drop(&mut self) {
        self.info.satp.free_page_table();
    }
}

pub fn init(){
}

/// 创建一个基本进程用于避免其它进程清空时报错
pub fn start_init_process(){
    let mgr = super::get_task_mgr().unwrap();
    let id = mgr.create_task(init_process as usize, true).unwrap();
    mgr.start(id, 0);
    panic!("start init process fail {}", 0);
}

/// 初始化进程
pub fn init_process(){
    // if fork() != 0 {
    //     device::run_interrupt();
    // }
    timer::set_next_interrupt(0);
    filesystem::init();
    let rt = fork();
    if rt != 0 {
        console_shell::run();
    }
    // if gpu_support() && fork() != 0 {
    //     println!("support gpu, start desktop!");
    //     let mut desk = Plane::new();
    //     desk.run();
    // }
    // if fork() != 0{
    //     buffer::write_down_handler();
    // }

    unsafe {
        loop {
            asm!("wfi");
        }
    }
}

extern crate alloc;
use crate::{
    filesystem,
    interact::console_shell,
    interrupt::timer,
    libs::syscall::fork,
    memory::map::SATP
};

use super::task_info::{ProgramInfo, TaskState};
use tisu_sync::AtomCounter;