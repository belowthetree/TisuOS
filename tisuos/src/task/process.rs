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
/// 保存环境、程序区域、堆区域（预计增加优先级）
/// heap、program 以虚拟地址进行交互，负责进程的映射工作
pub struct Process{
    pub info : ProgramInfo,
    pub tid : Vec<usize>,
    heap : TaskHeap,
    program : ProgramArea,
    resource : Resource,
    pub is_kernel : bool,
}

impl Process {
    /// ## 新建一个进程
    /// 进程仅进行基本信息的初始化：堆链表、satp
    pub fn new(program : ProgramArea) -> Option<Self> {
        let pid = unsafe {PID_CNT.add()};
        let info = ProgramInfo{
            pid,
            satp : SATP::new(),
            state: TaskState::Running,
            is_kernel : program.is_kernel,
        };
        program.map(&info.satp);
        let rt = Process{
            info,
            is_kernel : program.is_kernel,
            heap : TaskHeap::new(unsafe {MEMORY_END}, program.is_kernel),
            program,
            resource:Resource::new(pid),
            tid : Vec::<usize>::new(),
        };
        Some(rt)
    }

    pub fn get_prog_info(&self)->ProgramInfo {
        self.info
    }

    pub fn entry(&self)->usize {
        self.program.entry()
    }

    pub fn alloc_heap(&mut self, size : usize)->(usize, usize) {
        assert!(self.info.satp.is_map());
        self.heap.alloc(size, &self.info.satp)
    }

    pub fn free_heap(&mut self, addr : usize) {
        self.heap.free(addr);
    }

    pub fn virt_to_phy(&self, va:usize)->usize {
        if va >= unsafe {MEMORY_END} {
            self.heap.virt_to_phy(va)
        }
        else if self.program.contain(va) {
            self.program.virt_to_phy(va)
        }
        else {
            va
        }
    }

    pub fn push_file(&mut self, id:usize) {
        self.resource.push_file(id);
    }

    pub fn release_file(&mut self, id:usize) {
        self.resource.release_file(id);
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
    let mut program = ProgramArea::new(init_process as usize, true);
    program.push_area(Area::kernel_code());
    program.push_area(Area::kernel_data());
    program.push_area(Area::virtio_area());
    program.push_area(Area::timer_area());
    let id = mgr.create_task(program).unwrap();
    mgr.start(id, 0);
    panic!("start init process fail {}", 0);
}

/// 初始化进程
pub fn init_process(){
    timer::set_next_interrupt(0);
    filesystem::init();
    let rt = fork();
    if rt == 0 {
        console_shell::run();
    }

    branch(output_handler as usize, 0, 0);

    if gpu_support() && fork() != 0 {
        println!("support gpu, start desktop!");
        let mut desk = Plane::new();
        desk.run();
    }
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
use crate::{desktop::plane::Plane, filesystem, interact::{console_input::output_handler, console_shell}, interrupt::timer, libs::syscall::{branch, fork}, memory::{Area, ProgramArea, config::MEMORY_END, heap_memory::TaskHeap, map::SATP}, virtio::device::gpu_support};

use super::{resource::Resource, task_info::{ProgramInfo, TaskState}};
use tisu_sync::AtomCounter;