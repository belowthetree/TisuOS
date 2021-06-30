use super::{environment::{Environment, Register}, timer};

const GET_HARTID : usize = 3;
const TRIGGER_TIMER : usize = 2;
const TRIGGER_SOFTWARE : usize = 1;

extern "C" {
    fn s_trap_vector();
}

pub fn handler(env : &mut Environment)->usize {
    let num = env.a0();
    match num {
        GET_HARTID => {
            env.regs[Register::A0.val()] = riscv64::mhartid::read();
            env.epc + 4
        }
        TRIGGER_TIMER => {
            timer::set_next_timer();
            riscv64::mip::clear(riscv64::sip::STIP);
            riscv64::mie::set(riscv64::MTIE);
            env.epc + 4
        }
        TRIGGER_SOFTWARE => {
            riscv64::mip::clear(riscv64::sip::SSIP);
            riscv64::mie::set(riscv64::MSIE);
            env.epc + 4
        }
        _ => {
            riscv64::scause::write(riscv64::mcause::read());
            riscv64::sepc::write(env.epc);
            riscv64::sstatus::clear(1 << 1);
            riscv64::sstatus::set(1 << 5);
            riscv64::sstatus::set(riscv64::mstatus::spie() | riscv64::mstatus::upie());
            let spp = riscv64::mstatus::mpp();
            riscv64::mstatus::set(riscv64::MPP);
            if spp != 0 {
                riscv64::sstatus::set(1 << 8);
            }
            s_trap_vector as usize
        }
    }
}