//! # 进程管理
//! 管理进程，每一个核心进程队列的首元素默认为正在执行的进程，为了安全性，目前只允许进入中断对进程进行操作
//! 进程的调度按顺序进行，暂未添加优先级等
//! 2020年12月12日 zg

use alloc::{prelude::v1::*};
use virtio::device;

extern "C" {
    fn switch_kernel_process(env : *mut u8) -> usize;
    fn process_exit();
}
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
impl ProcessState {
    pub fn to_task_state(&self)->TaskState {
        match self {
            ProcessState::Scheduling => {TaskState::Running}
            ProcessState::Sleeping => {TaskState::Sleeping}
        }
    }
}
impl PartialEq for ProcessState{
    fn eq(&self, other: &Self) -> bool {
        (self.clone() as usize) == (*other as usize)
    }
}
/// ## 进程信息结构体
/// 保存环境、栈堆信息（预计增加唤醒时间、优先级）
/// pages 属于进程占用的物理页
pub struct Process{
    pub satp : usize,
    pub heap_list : *mut MemoryList,
    pub state : ProcessState,
    pub hartid : usize,
    pub pid : usize,
    pub tid : Vec<usize>,
    pub is_kernel : bool,
}

impl Process {
    /// ## 新建一个进程
    /// 内核或用户，传入的是虚拟地址，初始仅映射单页的代码页及全部栈页，堆页暂时不映射
    pub fn new(func : usize, is_kernel : bool) -> Option<Self> {
        let pid = unsafe {PID_CNT};
        let heap = MemoryList::new(HEAP_SIZE, is_kernel);
        if heap.is_null(){
            unsafe {(*heap).free(is_kernel);}
            None
        }
        else {
            let pt = page_table::PageTable::new();
            // 页表映射部分
            // 这一步极为重要，将内核中的一些函数（process_exit）进行映射
            // 对于内核权限的进程而言，还需要获取整个内存的访问权限
            page_table::map_kernel_area(&mut *pt, is_kernel);
            // 代码、栈的映射
            if !is_kernel{
                pt.map_user(func as usize, func as usize);
            }
            else {
                pt.map_kernel(func as usize, func as usize);
            }
            let satp = page_table::make_satp(pt as *mut PageTable as usize, 0);

            unsafe {
                PID_CNT = PID_CNT % PROCESS_NUM_MAX;
                PID_CNT += 1;
            }

            let mut rt = Process{
                    satp : satp,
                    pid : pid,
                    heap_list : heap,
                    state : ProcessState::Scheduling,
                    is_kernel : is_kernel,
                    tid : Vec::<usize>::new(),
                    hartid : 0
                };
            Some(rt)
        }
    }
    /// ## 利用现有的 satp 创建进程
    /// 默认已经映射代码页，不再映射代码
    pub fn from_satp(func : usize, satp : usize, is_kernel : bool)->Option<Self>{
        let pid = unsafe {PID_CNT};
        // 根据是否内核权限进行堆、栈内存申请
        let heap = MemoryList::new(HEAP_SIZE, is_kernel);
        if heap.is_null(){
            unsafe {(*heap).free(is_kernel);}
            None
        }
        //
        else {
            let t = SATP::from(satp);
            let pt = t.get_page_table();
            if pt.is_none(){
                return None;
            }
            let pt = pt.unwrap();
            // 页表映射部分
            // 这一步极为重要，将内核中的一些函数（process_exit）进行映射
            // 对于内核权限的进程而言，还需要获取整个内存的访问权限
            page_table::map_kernel_area(&mut *pt, is_kernel);

            let satp = page_table::make_satp(pt as *mut PageTable as usize, 0);
            unsafe {
                PID_CNT = PID_CNT % PROCESS_NUM_MAX;
                PID_CNT += 1;
            }
            let mut rt = Process{
                satp : satp,
                pid : pid,
                heap_list : heap,
                state : ProcessState::Scheduling,
                is_kernel : is_kernel,
                tid : Vec::<usize>::new(),
                hartid : 0
            };
            // if rt.fork(func).is_err(){
            //     None
            // }else{
            //     Some(rt)
            // }
            None
        }
    }

    pub fn drop_thread(&mut self, tid : usize){
        for (idx, th) in self.tid.iter().enumerate() {
            if *th == tid{
                self.tid.remove(idx);
                break;
            }
        }
    }

    pub fn get_prog_info(&self)->ProgramInfo {
        ProgramInfo::from_program(self)
    }
}
/// 进程的释放发生在被从调度队列中剔除
impl Drop for Process{
    fn drop(&mut self) {
        SATP::from(self.satp).free_page_table();
        unsafe {(*self.heap_list).free(self.is_kernel);}
    }
}

static mut TMP_ENV : [*mut Environment;4] = [null_mut();4];
pub static mut PID_CNT : usize = 1;
pub static STACK_PAGE_NUM : usize = 16;
pub static mut PROCESS_LIST : Option<VecDeque::<Process>> = None;
pub static HEAP_SIZE : usize = 4096;
pub static PROCESS_NUM_MAX : usize = 100000;
static mut PROCESS_LIST_LOCK : Mutex = Mutex::new();

pub fn init(){
    println!("init process");
    unsafe {
        for i in 0..4{
            TMP_ENV[i] = allocator::alloc(size_of::<Environment>(), true).unwrap() as *mut Environment;
        }
        PROCESS_LIST = Some(VecDeque::new());
    }
}

///
/// ## 机器模式（中断）调用部分，严禁用户、内核权限下调用
/// 

/// 创建一个基本进程用于避免其它进程清空时报错
pub fn start_init_process(){
    println!("start init process");
    let mgr = super::get_task_mgr().unwrap();
    println!("before create task");
    let id = mgr.create_task(init_process as usize, true).unwrap();
    println!("after create task {}", id);
    mgr.start(id);
    panic!("after schdule {}", 0);
}

/// 初始化进程
pub fn init_process(){
    if fork() != 0 {
        device::run_interrupt();
    }
    timer::set_next_interrupt(0);
    virtio::init();
    filesystem::init();
    gpu_device::reset(0);
    let rt = fork();
    if rt != 0 {
        println!("fork shell rt {}", rt);
        console_shell::run();
    }
    if fork() != 0 {
        let mut desk = Plane::new();
        desk.run();
    }
    // if fork() != 0{
    //     buffer::write_down_handler();
    // }
    
    unsafe {
        loop {
            asm!("wfi"::::"volatile");
        }
    }
}

extern crate alloc;
use crate::{desktop::plane::Plane, filesystem, interact::console_shell, interrupt::timer, libs::syscall::{fork}, memory::{allocator, page_table::PageTable, user_allocator::MemoryList}, sync::Mutex, virtio::{self, gpu_device}};
// use syscall::fork;
use core::{mem::size_of, ptr::null_mut};
use alloc::{collections::VecDeque};
use page_table::{SATP};
use crate::{interrupt::trap::{Environment, Register},memory::page_table, uart, };

use super::{info::ProgramInfo, task_manager::TaskState, thread::{self, Thread}};