//! # Trap
//! 中断管理部分，包括环境结构，各类中断处理
//! 2020年11月 zg

const INSTRUCTION_ADDRESS_MISALIGNED    : usize = 0;
const INSTRUCTION_ACCESS_FAULT          : usize = 1;
const ILLEGAL_INSTRUCTION               : usize = 2;
const BREAKPOINT                        : usize = 3;
const LOAD_ACCESS_FAULT                 : usize = 5;
const STORE_ADDRESS_MISALIGNED          : usize = 6;
const STORE_ACCESS_FAULT                : usize = 7;
const USER_ENVIRONMENT_CALL             : usize = 8;
const SUPERVISOR_ENVIRONMENT_CALL       : usize = 9;
const MACHINE_ENVIRONMENT_CALL          : usize = 11;
const INSTRUCTION_PAGE_FAULT            : usize = 12;
const LOAD_PAGE_FAULT                   : usize = 13;
const STORE_PAGE_FAULT                  : usize = 15;
const MACHINE_SOFTWARE                  : usize = 3;
const SUPERVISOR_SOFTWARE               : usize = 1;
const SUPERVISOR_TIMER                  : usize = 5;
const MACHINE_TIMER                     : usize = 7;
const SUPERVISOR_EXTERNAL               : usize = 9;
const MACHINE_EXTERNAL                  : usize = 11;

global_asm!(include_str!("../asm/func.S"));

pub static mut ENVS : [Environment;4] = [Environment::new();4];

pub fn init(hartid : usize){
    unsafe {
        let ad = (&mut ENVS[hartid] as *mut Environment) as usize;
        cpu::write_mscratch(ad);
    }
}

extern "C" {
    pub fn waiting();
}

#[no_mangle]
extern "C" fn m_trap(env:&mut Environment, cause:usize,
        hartid:usize, status : usize, _sp : usize, mtval : usize) -> usize{
    let sync;
    if (cause >> 63) & 1 == 1 {
        sync = false;
    }
    else {
        sync = true;
    }

    let num = cause & 0xfff;

    if sync {
        if num != 8 && num != 9 && num != 11 && num != STORE_PAGE_FAULT && num != LOAD_PAGE_FAULT {
            println!("into m_trap cause: {:x}, hartid: {:x}, status: {:x}, epc: {:x}, sp: {:x} st:{:x} ed:{:x},
                satp {:x}, mscratch {:x}, mtval {:x}",
                cause, hartid, status, env.epc, env.regs[Register::SP.val()],
                unsafe {KERNEL_STACK_START}, unsafe {KERNEL_STACK_END}, env.satp,
                (env as *const Environment) as usize, mtval);
            if let Some((e, p)) =
                get_task_mgr().unwrap().get_current_task(hartid) {
                    println!("{:x?}\n{:x?}", e, p);
            }
        }
        match num {
            INSTRUCTION_ADDRESS_MISALIGNED => panic!("Instruction address misaligned"),
            INSTRUCTION_ACCESS_FAULT => panic!("Instruction access fault"),
            ILLEGAL_INSTRUCTION => {
                illegal_instruction(env, mtval);
            },
            BREAKPOINT => {
                println!("Breakpoint");
                env.epc += 2;
            }
            LOAD_ACCESS_FAULT => panic!("Load access fault"),
            STORE_ADDRESS_MISALIGNED => panic!("Store address misalign"),
            STORE_ACCESS_FAULT => panic!("Store access fault"),
            MACHINE_ENVIRONMENT_CALL => {
                env.epc += 4;
            },
            SUPERVISOR_ENVIRONMENT_CALL|USER_ENVIRONMENT_CALL=> {
                match syscall::handler(env) {
                    syscall::SyscallResult::Schedule(_) => {
                        let mgr = get_task_mgr().unwrap();
                        env.epc += 4;
                        mgr.schedule(env);
                        write_satp(0);
                        env.epc = waiting as usize;
                    },
                    syscall::SyscallResult::Normal(rt) => {
                        env.regs[Register::A0.val()] = rt;
                        env.epc += 4;
                    },
                }
            }
            INSTRUCTION_PAGE_FAULT => instruction_page_fault(env, mtval),
            LOAD_PAGE_FAULT => load_page_fault(status, mtval, env),
            STORE_PAGE_FAULT => store_page_fault(status, mtval, env),
            _ => panic!("unhandled sync number: {:016x}", num),
        }
    }
    else {
        match num {
            // 软件中断
            SUPERVISOR_SOFTWARE => println!("supervisor software"),
            MACHINE_SOFTWARE => {
                software::inactivate(hartid);
                get_task_mgr().unwrap().schedule(env);
                write_satp(0);
                env.epc = waiting as usize;
            },
            SUPERVISOR_TIMER => println!("supervisor timer interrupt"),
            MACHINE_TIMER => {
                timer::set_next_timer();
                activate_hart();
                get_task_mgr().unwrap().schedule(env);
                write_satp(0);
                env.epc = waiting as usize;
            },
            SUPERVISOR_EXTERNAL => println!("supervisor external"),
            MACHINE_EXTERNAL => {
                plic::handler();
            }
            _ => {
                panic!("unknown interrupt number: {:016x}", num);
            }
        }
    }
    env.epc
}

fn illegal_instruction(env:&mut Environment, mtval : usize) {
    let satp = SATP::from(env.satp);
    let addr = satp.get_target(env.epc);
    println!("epc {:x}, mtval {:x} target pa {:x} val {:x}", env.epc,
        mtval, addr, unsafe{((addr + env.epc % PAGE_SIZE) as *const u32).read_volatile()});
    let mgr = get_task_mgr().unwrap();
    let (e, _) = mgr.get_current_task(env.hartid).unwrap();
    if e.is_kernel {
        panic!("pid {} tid {} illegal instruction", e.pid, e.tid);
    }
    else {
        drop_task(e.tid, env);
    }
}

fn activate_hart() {
    software::activate(1);
    software::activate(2);
    software::activate(3);
}

fn instruction_page_fault(env:&mut Environment, mtval : usize) {
    let satp = SATP::from(env.satp);
    let addr = satp.get_target(env.epc);
    println!("epc {:x}, mtval {:x} target pa {:x} val {:x}", env.epc,
        mtval, addr, unsafe{((addr + env.epc % PAGE_SIZE) as *const u32).read_volatile()});
    let mgr = get_task_mgr().unwrap();
    let (e, _) = mgr.get_current_task(env.hartid).unwrap();
    if e.is_kernel{
        panic!("pid {} tid {} Instruction page fault", e.pid, e.tid);
    }
    else {
        drop_task(e.tid, env);
    }
}

fn load_page_fault(status : usize, mtval : usize, env:&mut Environment) {
    println!("load page fault");
    let mgr = get_task_mgr().unwrap();
    let id =mgr.get_current_task(env.hartid).unwrap().0.tid;
    if mgr.expand_stack(id).is_err() {
        println!("into m_trap hartid: {:x}, status: {:x}, epc: {:x}, sp: {:x} st:{:x} ed:{:x},
            satp {:x}, mscratch {:x}, mtval {:x}",
            env.hartid, status, env.epc, env.regs[Register::SP.val()],
            unsafe {KERNEL_STACK_START}, unsafe {KERNEL_STACK_END}, env.satp,
            (env as *const Environment) as usize, mtval);
        let (e, _) = get_task_mgr().unwrap().get_current_task(env.hartid).unwrap();
        if e.is_kernel {
            panic!("pid {} tid {} load page fault", e.pid, e.tid);
        }
        else {
            drop_task(e.tid, env);
        }
    }
}

fn store_page_fault(status : usize, mtval : usize, env:&mut Environment) {
    println!("store page fault");
    let mgr = get_task_mgr().unwrap();
    let id =mgr.get_current_task(env.hartid).unwrap().0.tid;
    if mgr.expand_stack(id).is_err() {
        println!("into m_trap hartid: {:x}, status: {:x}, epc: {:x}, sp: {:x} st:{:x} ed:{:x},
            satp {:x}, mscratch {:x}, mtval {:x}",
            env.hartid, status, env.epc, env.regs[Register::SP.val()],
            unsafe {KERNEL_STACK_START}, unsafe {KERNEL_STACK_END}, env.satp,
            (env as *const Environment) as usize, mtval);
        let (e, _) = get_task_mgr().unwrap().get_current_task(env.hartid).unwrap();
        if e.is_kernel {
            panic!("pid {} tid {} store page fault", e.pid, e.tid);
        }
        else {
            drop_task(e.tid, env);
        }
    }
}

fn drop_task(id:usize, env:&mut Environment) {
    println!("drop task {}", id);
    let mgr = get_task_mgr().unwrap();
    mgr.program_exit(id);
    activate_hart();
    mgr.schedule(env);
    write_satp(0);
    env.epc = waiting as usize;
}

use crate::{interrupt::{environment::Register, software}, libs::cpu::write_satp, memory::{config::{KERNEL_STACK_END, KERNEL_STACK_START, PAGE_SIZE}, map::SATP}, task::get_task_mgr};
use crate::{plic, cpu};
use super::{environment::Environment, syscall, timer};