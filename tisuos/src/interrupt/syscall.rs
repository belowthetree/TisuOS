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
            rt = exec(env.regs[Register::A1.val()], env.regs[Register::A2.val()] != 0);
        }
        5 => {
            // 输出任务信息
        }
        6 => {
        }
        7 => {
            branch(env);
        }
        57 => {
            fork(env);
        }
        60 => {
            println!("delete process");
            let mgr = get_task_mgr().unwrap();
            mgr.program_exit(env.hartid);
            mgr.schedule(&env);
        }
        61 => {
            println!("delete thread");
            let mgr = get_task_mgr().unwrap();
            mgr.task_exit(env.hartid);
            mgr.schedule(&env);
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

fn exec(func : usize, is_kernel : bool)->usize {
    let mgr = get_task_mgr().unwrap(); 
    let id = mgr.create_task(func, is_kernel).unwrap();
    id
}



use crate::{task::task_manager::TaskState};
use crate::uart;
use crate::task::get_task_mgr;
use super::trap::{Environment, Register};