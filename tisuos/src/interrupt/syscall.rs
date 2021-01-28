//! # System call
//! 系统调用转到这里处理
//! 2020年12月18日 zg

pub fn handler(env : &Environment) -> usize {
    let mut rt = 0;
    let num = env.regs[Register::A0.val()];
    match num {
        1 => {
            println!("syscall test");
        }
        2 => {
            // 设置 timer 触发
        }
        3 => {
            panic!("shei gan diaoyong {}", 0);
        }
        4 => {
            exec(env.regs[Register::A1.val()], env.regs[Register::A2.val()],
            env.regs[Register::A3.val()] != 0);
        }
        5 => {
            process::print();
            thread::print();
        }
        6 => {
            if let Some(addr) = allocator::alloc(env.regs[Register::A1.val()], true) {
                rt = addr as usize;
            }
        }
        57 => {
            let t = get_current_thread(env.hartid).unwrap();
            fork(env, t);
        }
        60 => {
            println!("delete process");
            process::delete_current_process(env.hartid);
            thread::schedule(env);
        }
        61 => {
            thread::delete_current_thread(env.hartid);
            thread::schedule(env);
        }
        _ => {}
    }
    rt
}

fn fork(env : &Environment, thread : &Thread){
    thread::fork(env, thread);
}
#[allow(dead_code)]
fn branch(func : usize, pid : usize)->Result<(), ()>{
    let p = get_process_by_pid(pid).unwrap();
    p.fork(func)
}

fn exec(func : usize, satp : usize, is_kernel : bool){
    if let Some(p) = Process::from_satp(func, satp, is_kernel){
        add_process(p);
    }
}

use process::{add_process, get_process_by_pid};
use thread::{Thread};

use crate::{memory::allocator, task::{process::{Process, self}, thread::{self, get_current_thread}}};
use crate::uart;

use super::trap::{Environment, Register};