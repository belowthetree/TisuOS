#![allow(dead_code)]
//! # ELF
//! 提供 ELF64 文件的操作接口
//! 2020年12月22日 zg

const MAGIC : u32 = 0x464c457f;

/// ## load elf
/// 加载 ELF 文件并创建进程执行，暂时使用
pub fn load_elf(file : File) {
    unsafe{
        let mut file = file;
        if file.is_close() && file.open_flag(OpenFlag::Read.val()).is_err(){
            return;
        }
        let head;
        if let Some(f) =file.read(0, size_of::<ELF>()){
            head = f;
        }
        else{
            return;
        }
        let elf = head.type_as::<ELF>();
        if elf.is_elf(){
            let phsize = elf.get_program_header_size();
            let program_offset = elf.get_program_header_addr();
            let num = elf.program_header_num as usize;
            let entry = elf.entry as usize;
            let id = exec(entry);
            let mgr = crate::task::get_task_mgr().unwrap();
            for i in 0..num{
                let offset = program_offset + i * phsize;
                let phr = file.read(offset, phsize).unwrap();
                let program = &*(phr.get_addr() as *const ProgramHeader);
                if program.is_loadable(){
                    let virtual_addr = program.virtual_addr as usize;
                    let mem_size = program.segment_size_in_memory as usize;
                    let file_size = program.segment_size_in_file as usize;
                    let num_page = (mem_size + PAGE_SIZE - 1) / PAGE_SIZE;
                    let physic_addr = alloc_user_page(num_page) as usize;
                    if let Some(buffer) =
                            file.read(program.offset_in_file as usize, file_size){
                        (buffer.get_addr() as *mut u8).copy_to(physic_addr as *mut u8, file_size);
                        for n in 0..num_page{
                            let virtual_addr = virtual_addr + n * PAGE_SIZE;
                            let physic_addr = physic_addr + n * PAGE_SIZE;
                            mgr.map_code(id, virtual_addr, physic_addr);
                        }
                    }
                    else{
                        mgr.program_exit(id);
                        return;
                    }
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ELF{
    identity : u64,
    identity2 : u64,
    etype : u16,
    machine : u16,
    version : u32,
    entry : u64,
    program_header_offset : u64,
    section_header_offset : u64,
    flags : u32,
    this_header_size : u16,
    program_header_size : u16,
    program_header_num : u16,
    section_header_size : u16,
    section_header_num : u16,
    section_str_table_index : u16,
}
impl ELF{
    pub fn is_elf(&self)->bool {
        let p = self as *const ELF as *const u32;
        unsafe {
            p.read_volatile() == MAGIC
        }
    }
    pub fn list(&self){
        println!("{:x?}", self);
    }
    pub fn get_section_header_addr(&self)->usize{
        self.section_header_offset as usize
    }
    pub fn get_program_header_addr(&self)->usize{
        self.program_header_offset as usize
    }
    pub fn get_program_header_size(&self)->usize{
        self.program_header_size as usize
    }
    pub fn get_section_size(&self)->usize{
        self.section_header_size as usize
    }
}

#[repr(packed)]
#[derive(Debug, Clone, Copy)]
pub struct SectionHeader{
    section_name : u32,
    stype : u32,
    flag : u64,
    virtual_addr : u64,
    offset_in_file : u64,
    section_size : u64,
    link_section : u32,
    info : u32,
    address_align : u64, // 地址对齐边界？？
    entry_size : u64,
}
impl SectionHeader {
    pub fn list(&self) {
        println!("{:x?}", self);
    }
}
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
pub struct ProgramHeader{
    ptype : u32,
    flags : u32,
    offset_in_file : u64,
    virtual_addr : u64,
    physic_addr : u64, // 保留
    segment_size_in_file : u64,
    segment_size_in_memory : u64,
    align : u64,
}
impl ProgramHeader {
    pub fn list(&self){
        println!("{:x?}", self);
    }
    pub fn is_loadable(&self)->bool {
        self.ptype == PType::Load.val()
    }
    pub fn is_null(&self)->bool {
        self.ptype == PType::Null.val()
    }
}

pub enum PType{
    Null = 0, // unused entry
    Load = 1, // 可加载
    Dynamic = 2,
    Interp = 3,
    Note = 4,
}
impl PType {
    pub fn val(self)->u32{
        self as u32
    }
}
#[derive(Debug)]
#[repr(u32)]
pub enum Etype{
    None = 0,
    Rel = 1,
    Exec = 2,
    Dynamic = 3,
    Core = 4,
    LoPro = 0xff00,
    HiPro = 0xffff,
}
pub enum Machine{
    None = 0,
    M32 = 1, // AT&T WE 32100
    Sparc = 2,
    Intel = 3, // Intel Arch
    Moto68 = 4, // 摩托罗拉 6800
    Moto88 = 5, // 摩托罗拉 8800
    Intel8086 = 7,
    Mips = 8, // MIPS RS3000大端
    Mips4 = 10, // MIPS RS4000 大端
}

use core::mem::size_of;

use crate::{libs::syscall::exec, memory::{page::{PAGE_SIZE, alloc_user_page}, page_table::{self, PageTable}}, uart};

use super::filetree::file::{File, OpenFlag};