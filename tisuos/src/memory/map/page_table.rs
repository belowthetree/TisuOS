use tisu_memory::MemoryOp;

use crate::memory::get_manager;

use super::pagebit::PageBit;

#[derive(Copy, Clone)]
pub struct PTE{
    flag : u64
}

impl PTE {
    pub fn is_valid(&self) -> bool {
        self.flag & PageBit::Valid.val() != 0
    }
    pub fn set_node_ppn(&mut self, ppn : u64){
        self.flag = (self.flag & 0x3ff) | ((ppn >> 12) << 10);
    }
    pub fn set_leaf_ppn(&mut self, ppn : u64){
        self.flag = (self.flag & 0x3ff) | ((ppn >> 12) << 10);
    }
    pub fn get_ppn(&self) -> usize {
        ((self.flag & !0x3ff) << 2) as usize
    }
    pub fn set_flag(&mut self, flag : u64){
        self.flag |= flag;
    }
    pub fn set_valid(&mut self){
        self.flag |= PageBit::Valid.val();
    }
}


/// ## 页表
/// Rv39 页表，本身占据一个页表（默认4KB）的大小
/// 分为三级，页表地址必须以一个页表的大小对齐
pub struct PageTable{
    entry : [PTE; 512],
}

impl PageTable {
    pub fn new() ->&'static mut Self {
        let addr = get_manager().kernel_page(1).unwrap();
        unsafe {
            &mut *(addr as *mut Self)
        }
    }

    pub fn map_kernel(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val()
            | PageBit::Excute.val());
    }

    pub fn map_kernel_data(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val());
    }

    pub fn map_kernel_code(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Excute.val());
    }

    pub fn map_user_data(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::Write.val() |
            PageBit::User.val());
    }

    pub fn map_user_code(&mut self, virtual_addr : usize, physic_addr : usize){
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::User.val()
        | PageBit::Excute.val());
    }

    pub fn map_user(&mut self, virtual_addr : usize, physic_addr : usize) {
        self.map(virtual_addr, physic_addr, PageBit::Read.val() | PageBit::User.val()
        | PageBit::Excute.val() | PageBit::Write.val());
    }

    fn map(&mut self, virtual_addr : usize, physic_addr : usize, flag : u64){
        let vpn = [
            (virtual_addr >> 30) & 0x1ff,
            (virtual_addr >> 21) & 0x1ff,
            (virtual_addr >> 12) & 0x1ff
        ];
        let pte_first = &mut self.entry[vpn[0]];
        if !pte_first.is_valid() {
            let addr = get_manager().kernel_page(1).unwrap();
            pte_first.set_node_ppn(addr as u64);
            pte_first.set_flag(flag &
                !PageBit::Read.val() & !PageBit::Write.val() & !PageBit::Excute.val());
            pte_first.set_valid();
        }
        let table_mid = unsafe {&mut *(pte_first.get_ppn() as *mut Self)};
        let pte_mid = &mut table_mid.entry[vpn[1]];
        
        if !pte_mid.is_valid() {
            let addr = get_manager().kernel_page(1).unwrap();
            pte_mid.set_node_ppn(addr as u64);
            pte_mid.set_flag(flag &
                !PageBit::Read.val() & !PageBit::Write.val() & !PageBit::Excute.val());
            pte_mid.set_valid();
        }
        let table_final = unsafe {&mut *(pte_mid.get_ppn() as *mut Self)};
        let pte_final = &mut table_final.entry[vpn[2]];
        pte_final.flag = 0;
        pte_final.set_leaf_ppn(physic_addr as u64);
        pte_final.set_flag(flag);
        pte_final.set_valid();
    }

    pub fn free(&mut self){
        for i in 0..512{
            let pte = &self.entry[i];
            if pte.is_valid(){
                let next_table = unsafe{&*(pte.get_ppn() as *mut PageTable)};
                for j in 0..512{
                    let pte = next_table.entry[j];
                    if pte.is_valid(){
                        // let next_table = unsafe{&*(pte.get_ppn() as *mut PageTable)};
                        // for k in 0..512{
                        //     let pte = next_table.entry[k];
                        //     if pte.is_valid(){
                        //         println!("pte {:x}", pte.get_ppn());
                        //         get_manager().free_page(pte.get_ppn() as *mut u8);
                        //     }
                        // }
                        get_manager().free_page(pte.get_ppn() as *mut u8);
                    }
                }
                get_manager().free_page(pte.get_ppn() as *mut u8);
            }
        }
        let addr = self as *mut Self;

        get_manager().free_page(addr as *mut u8);
    }
}