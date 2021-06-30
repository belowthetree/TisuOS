//! # 进程
//! 进程作为程序标志，为程序保存基本信息
//!
//! 2020年12月12日 zg

use alloc::{prelude::v1::*};

pub static mut PID_CNT : AtomCounter = AtomCounter::new();
pub const MAX_HEAP_SIZE : usize = PAGE_SIZE * 1024 * 4;


/// ## 进程信息结构体
/// 保存环境、程序区域、堆区域
/// heap、program 以虚拟地址进行交互，负责进程的映射工作
pub struct Process{
    pub info : ProgramInfo,
    pub tid : Vec<usize>,
    heap : TaskHeap,
    program : ProgramArea,
    resource : Resource,
    pub join_num : usize,
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
        program.map_kernel_trap(&info.satp);
        let rt = Process{
            info,
            is_kernel : program.is_kernel,
            heap : TaskHeap::new(unsafe {MEMORY_END}, program.is_kernel),
            program,
            resource:Resource::new(pid),
            join_num : 0,
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

    pub fn contain(&self, va:usize)->bool {
        if va >= unsafe {MEMORY_END} && va <= MAX_HEAP_SIZE + unsafe {MEMORY_END} {
            true
        }
        else if self.program.contain(va) {
            true
        }
        else {
            false
        }
    }

    pub fn virt_to_phy(&self, va:usize)->usize {
        if va >= unsafe {MEMORY_END} {
            if va <= MAX_HEAP_SIZE + unsafe {MEMORY_END} {
                self.heap.virt_to_phy(va)
            }
            else {
                panic!("virt to phy va {:x}", va);
            }
        }
        else if self.program.contain(va) {
            self.program.virt_to_phy(va)
        }
        else {
            va
        }
    }

    /// 添加文件 ID
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
    program.push_area(Area::rtc_area());
    program.push_area(Area::test_area());
    let id = mgr.create_task(program, &Environment::new()).unwrap();
    mgr.start(id, 0);
    panic!("start init process fail {}", 0);
}

/// 初始化进程
pub fn init_process(){
    disk_cache::init();
    timer::set_next_interrupt(0);
    filesystem::init();
    if fork() == 0 {
        console_shell::run();
    }

    branch(output_handler as usize, 0, 0);

    if gpu_support() && fork() == 0 {
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
            // let mut num = 0;
            // for i in 0..100000 {
            //     num += 1;
            // }
            // println!("{}", num);
        }
    }
}


extern crate alloc;
use crate::{desktop::plane::Plane, filesystem, interact::{console_input::output_handler, console_shell}, interrupt::{environment::Environment, timer}, libs::syscall::{branch, fork}, memory::{Area, ProgramArea, config::{MEMORY_END, PAGE_SIZE}, heap_memory::TaskHeap, map::SATP}, virtio::{device::gpu_support, disk_cache}};

use super::{resource::Resource, task_info::{ProgramInfo, TaskState}};
use tisu_sync::AtomCounter;