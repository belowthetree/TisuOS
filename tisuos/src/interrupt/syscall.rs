//! # System call
//! 系统调用转到这里处理
//! 2020年12月18日 zg

const PROGRAM_EXIT  : usize = 60;
const THREAD_EXIT   : usize = 61;
const FORK          : usize = 57;
const BRANCH        : usize = 7;
const PRINT_TASK    : usize = 5;
const EXEC          : usize = 4;

pub fn handler(env : &Environment) -> usize {
    let mut rt = 0;
    let num = env.regs[Register::A0.val()];
    match num {
        1 => {
            println!("syscall test");
        }
        2 => {
            println!("syscall test2");
        }
        3 => {
            panic!("shei gan diaoyong {}", 0);
        }
        EXEC => {
            let ptr = env.regs[Register::A1.val()] as *mut char;
            let len = env.regs[Register::A2.val()];
            let path = unsafe {&*(slice_from_raw_parts(ptr, len))};
            rt = exec(char_to_str(path), env.regs[Register::A2.val()] != 0);
        }
        PRINT_TASK => {
            get_task_mgr().unwrap().print();
        }
        6 => {  
        }
        BRANCH => {
            branch(env);
        }
        FORK => {
            fork(env);
        }
        PROGRAM_EXIT => {
            println!("delete process");
            let mgr = get_task_mgr().unwrap();
            mgr.program_exit(env.hartid);
        }
        THREAD_EXIT => {
            println!("delete thread");
            let mgr = get_task_mgr().unwrap();
            mgr.task_exit(env.hartid);
        }
        _ => {}
    }
    rt
}

fn fork(env : &Environment){
    get_task_mgr().unwrap().fork_task(env);
}

fn branch(env : &Environment){
    let mgr = get_task_mgr().unwrap();
    let id = mgr.branch(env).unwrap();
    mgr.set_task_state(id, TaskState::Sleeping);
}

fn exec(path : String, _is_kernel: bool)->usize {
    let is_kernel = false;
    let idx = path.find("/").unwrap();
    let (id, p) = path.split_at(idx);
    let id = convert_to_usize(&id.to_string());
    let sys = get_system(id).unwrap();
    let file = sys.open(p.to_string(), tisu_fs::FileFlag::Read).unwrap().clone();
    let data = Block::<u8>::new(file.size);
    sys.read(file.id, data.to_array(0, file.size)).unwrap();
    let elf = data.type_as::<ELF>();
    let mut elf = ElfManager::new(elf);
    let mgr = get_task_mgr().unwrap();
    let task_id = mgr.create_task(elf.entry(), is_kernel).unwrap(); // 较慢
    let info = mgr.get_program_info(task_id);
    info.satp.map_elf(&mut elf, is_kernel);
    mgr.wake_task(task_id);

    0
}


use core::ptr::slice_from_raw_parts;

use alloc::prelude::v1::*;
use crate::{filesystem::{elf::{ELF, ElfManager}, get_system}, libs::str::{char_to_str, convert_to_usize}, memory::block::Block, task::task_info::TaskState};
use crate::task::get_task_mgr;
use super::trap::{Environment, Register};