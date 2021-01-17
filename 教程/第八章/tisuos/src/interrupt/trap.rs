//! # Trap
//! 中断管理部分，包括环境结构，各类中断处理
//! 2020年11月 zg
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
#[derive(Copy, Clone)]
pub struct Environment{
    pub regs    :     [usize;32], // 0 ~ 255
    fregs       :     [usize;32], // 256 ~ 511
    pub satp    :     usize,      // 512
    pub epc     :     usize,      // 520
}

impl Environment {
    pub const fn new()->Self{
        Environment{
            regs : [0;32],
            fregs :[0;32],
            satp:  0,
            epc:   0,
        }
    }
    pub fn init(&mut self, epc : usize, stack : usize, satp : usize){
        self.epc = epc;
        self.satp = satp;
        self.regs[2] = stack;
    }
    pub fn copy(&mut self, env : &Environment){
        for i in 0..32{
            self.regs[i] = env.regs[i];
            self.fregs[i] = env.fregs[i];
        }
        self.satp = env.satp;
        self.epc = env.epc;
    }
}

pub static mut ENVS : [Environment;4] = [Environment::new();4];

pub fn init(){
    unsafe {
        let ad = (&mut ENVS[cpu::get_hartid()] as *mut Environment) as usize;
        cpu::write_scratch(ad);
    }
}

#[no_mangle]
extern "C" fn m_trap(env:&mut Environment, cause:usize,
        hartid:usize, _status : usize, _sp : usize) -> usize{
    
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
        match num {
            0 => {
                panic!("Instruction address misaligned CPU:{:016x} at epc:{:016x}\n", 
                    hartid, pc);
            },
            1 => {
                panic!("Instruction access fault CPU:{:016x} at epc:{:016x}", hartid, pc);
            },
            2 => {
                panic!("Illegal instruction CPU:{:016x} at epc:{:016x}", hartid, pc);
            },
            3 => {
                println!("Breakpoint");
                pc += 2;
            },
            5 => {
                panic!("Load access fault CPU:{} at epc:{:016x}", hartid, pc);
            }
            7 => {
                panic!("Store access fault CPU:{:016x} at epc:{:016x}", hartid, pc);
            }
            8 | 9 | 11 => {
                // 系统调用
            },
            12 => {
                println!("Instruction page fault {}", 0);
            }
            13 => {
                println!("Load page fault");
            }
            15 => {
                println!("Store page fault {}", 0);
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
                println!("Machine software interrupt CPU:{:016x}", hartid);
            },
            5 => {
                println!("Machine timer interrupt");
            }
            7 => {
                timer::set_next_timer();
                // process::schedule(env, hartid);
                thread::schedule(env);
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

use crate::interrupt::timer;
use crate::task::thread;
use crate::plic;
use crate::{uart};
use crate::cpu;