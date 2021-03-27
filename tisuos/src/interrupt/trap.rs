//! # Trap
//! 中断管理部分，包括环境结构，各类中断处理
//! 2020年11月 zg
#[allow(dead_code)]
pub enum Register{
    RA = 1,
    SP = 2,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
}
impl Register {
    pub fn val(self)->usize{
        self as usize
    }
}
/// 保存需要恢复的环境
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Environment{
    pub regs    :     [usize;32], // 0 ~ 255
    fregs       :     [usize;32], // 256 ~ 511
    pub satp    :     usize,      // 512
    pub epc     :     usize,      // 520
    pub hartid  :     usize,      // 528
}

impl Environment {
    pub const fn new()->Self{
        Environment{
            regs : [0;32],
            fregs :[0;32],
            satp:  0,
            epc:   0,
            hartid: 0,
        }
    }
}

pub static mut ENVS : [Environment;4] = [Environment::new();4];
static mut LOCKER : Mutex = Mutex::new();

pub fn init(hartid : usize){
    unsafe {
        LOCKER.lock();
        let ad = (&mut ENVS[hartid] as *mut Environment) as usize;
        cpu::write_mscratch(ad);
        LOCKER.unlock();
    }
}

extern "C" {
    pub fn waiting();
}

#[no_mangle]
extern "C" fn m_trap(env:&mut Environment, cause:usize,
        hartid:usize, _status : usize, _sp : usize, mtval : usize) -> usize{
    let sync;
    if (cause >> 63) & 1 == 1 {
        sync = false;
    }
    else {
        sync = true;
    }

    let num = cause & 0xfff;
    let mut pc = env.epc;

    if sync {
        if num != 8 && num != 9 && num != 11{
            println!("into m_trap cause: {:x}, hartid: {:x}, status: {:x}, epc: {:x}, sp: {:x} st:{:x} ed:{:x},
                satp {:x}, mscratch {:x}, mtval {:x}",
                cause, hartid, _status, env.epc, env.regs[Register::SP.val()],
                unsafe {KERNEL_STACK_START}, unsafe {KERNEL_STACK_END}, env.satp,
                (env as *const Environment) as usize, mtval);
        }
        match num {
            0 => {
                panic!("Instruction address misaligned CPU:{:016x} at epc:{:016x}\n", 
                    hartid, pc);
            },
            1 => {
                panic!("Instruction access fault CPU:{:016x} at epc:{:016x}", hartid, pc);
            }
            2 => {
                panic!("Illegal instruction CPU:{:016x} at epc:{:016x}", hartid, pc);
            }
            3 => {
                println!("Breakpoint");
                pc += 2;
            }
            5 => {
                panic!("Load access fault CPU:{} at epc:{:016x}", hartid, pc);
            }
            6 => {
                panic!("Store address misalign {}", 0);
            }
            7 => {
                panic!("Store access fault CPU:{:016x} at epc:{:016x}", hartid, pc);
            }
            8 | 9 | 11 => {
                env.regs[Register::A0.val()] = syscall::handler(env);
                println!("syscall rt {}", env.regs[Register::A0.val()]);
                // env.epc = pc + 4;
                pc += 4;
                // thread::schedule(env);
                // pc = waiting as usize;
            }
            12 => {
                panic!("Instruction page fault {}", 0);
            }
            13 => {
                panic!("Load page fault");
            }
            15 => {
                panic!("Store page fault epc {:x}", env.epc);
            }
            _ => {
                println!("into m_trap cause: {:x}, hartid: {:x}, status: {:x}, epc: {:x}, sp: {:x}, satp {:x}",
                    cause, hartid, _status, env.epc, _sp, env.satp);
                panic!("unknown sync number: {:016x}", num);
            }
        }
    }
    else {
        match num {
            // 软件中断
            3 => {
                // println!("Machine software interrupt CPU:{:016x}", hartid);
                unsafe {
                    let ptr = 0x200_0000 as *mut u32;
                    ptr.add(hartid).write_volatile(0);
                }
                get_task_mgr().unwrap().schedule(env);
                // println!("core {} receive", hartid);
                pc = waiting as usize;
            },
            5 => {
                println!("Machine timer interrupt");
            }
            7 => {
                timer::set_next_timer();
                unsafe {
                    let ptr = 0x2000000 as *mut u32;
                    ptr.add(1).write_volatile(1);
                    ptr.add(2).write_volatile(1);
                    ptr.add(3).write_volatile(1);
                }
                get_task_mgr().unwrap().schedule(env);
                pc = waiting as usize;
            },
            11 => {
                plic::handler();
            }
            _ => {
                panic!("unknown interrupt number: {:016x}", num);
            }
        }
    }
    pc
}


use crate::{memory::{KERNEL_STACK_END, KERNEL_STACK_START}, sync::Mutex, task::{get_task_mgr}};

use crate::{plic, uart, cpu};
use super::{syscall, timer};