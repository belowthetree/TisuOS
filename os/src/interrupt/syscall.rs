//! # System call
//! 系统调用转到这里处理
//! 2020年12月18日 zg

pub fn handler(env : &Environment) -> usize {
    let mut rt = 0;
    let num = env.regs[Register::A0.val()];
    // println!("syscall {}", num);
    match num {
        1 => {
            println!("syscall test");
        }
        2 => {
            // 设置 timer 触发
        }
        3 => {
            rt = cpu::get_hartid();
        }
        4 => {
            exec(env.regs[Register::A1.val()], env.regs[Register::A2.val()],
            env.regs[Register::A3.val()] != 0);
        }
        5 => {
            process::print();
            thread::print();
        }
        57 => {
            let func = env.regs[Register::A1.val()];
            let pid = get_current_thread().unwrap().pid;
            if fork(func, pid).is_ok(){
                rt = 1;
            }
            else{
                rt = 0;
            }
        }
        60 => {
            process::delete_current_process();
            thread::schedule(env);
        }
        61 => {
            thread::delete_current_thread();
            thread::schedule(env);
        }
        _ => {}
    }
    rt
}

fn fork(func : usize, pid : usize)->Result<(), ()>{
    let p = get_process_by_pid(pid).unwrap();
    p.fork(func)
}

fn exec(func : usize, satp : usize, is_kernel : bool){
    if let Some(p) = Process::from_satp(func, satp, is_kernel){
        add_process_machine(p);
    }
}

use process::{add_process_machine, get_process_by_pid};

use crate::{cpu, task::{process::{Process, self}, thread::{self, get_current_thread}}};
use crate::uart;

use super::trap::{Environment, Register};