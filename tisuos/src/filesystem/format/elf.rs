#![allow(dead_code)]
//! # ELF
//! 提供 ELF64 文件的操作接口
//! 2020年12月22日 zg

const MAGIC : u32 = 0x464c457f;

pub struct ElfManager {
    elf : &'static ELF,
    ph_idx : usize,
    sh_idx : usize,
}

impl ElfManager {
    pub fn new(elf : &mut ELF)->Self {
        Self {
            elf : unsafe {&*(elf as *mut ELF)},
            ph_idx : 0,
            sh_idx : 0,
        }
    }

    pub fn reset(&mut self) {
        self.ph_idx = 0;
        self.sh_idx = 0;
    }

    pub fn entry(&self)->usize {
        self.elf.entry as usize
    }

    pub fn next_ph(&mut self)->Option<&'static ProgramHeader> {
        if self.ph_idx >= self.elf.program_header_num as usize {
            None
        }
        else {
            unsafe {
                let start = self.elf as *const ELF as *const u8;
                let offset = self.elf.program_header_offset as usize
                     + self.elf.program_header_size as usize * self.ph_idx;
                self.ph_idx += 1;
                Some(&*(start.add(offset) as *const ProgramHeader))
            }
        }
    }

    pub fn cur_ph(&self)->Option<&'static ProgramHeader> {
        if self.ph_idx >= self.elf.program_header_num as usize {
            None
        }
        else {
            unsafe {
                let start = self.elf as *const ELF as *const u8;
                let idx = self.ph_idx - 1;
                let offset = self.elf.program_header_offset as usize
                     + self.elf.program_header_size as usize * idx;
                Some(&*(start.add(offset) as *const ProgramHeader))
            }
        }
    }

    pub fn next_sh(&mut self)->Option<&'static SectionHeader> {
        if self.sh_idx >= self.elf.section_header_num as usize {
            None
        }
        else {
            unsafe {
                let start = self.elf as *const ELF as *const u8;
                let idx = self.elf.section_header_offset as usize
                     + self.elf.section_header_size as usize * self.sh_idx;
                self.sh_idx += 1;
                Some(&*(start.add(idx) as *const SectionHeader))
            }
        }
    }

    pub fn get_str(&self, idx : usize)->Option<String> {
        unsafe {
            let mut offset = idx;
            let start = self.elf as *const ELF as *const u8;
            for i in 0..self.elf.section_header_num as usize {
                let idx = self.elf.section_header_offset as usize
                     + self.elf.section_header_size as usize * i;
                let sh = &*(start.add(idx) as *const SectionHeader);
                if sh.stype == SType::StringTable {
                    if sh.section_size as usize <= offset {
                        offset -= sh.section_size as usize;
                    }
                    else {
                        let mut t = start.add(sh.offset_in_file as usize);
                        t = t.add(offset);
                        let mut s = String::new();
                        while *t != 10 {
                            print!("{}", *t as char);
                            s.push(*t as char);
                            t = t.add(1);
                        }
                        return Some(s);
                    }
                }
            }
        }
        None
    }

    pub fn get_addr(&self, offset : usize)->*const u8 {
        unsafe {(self.elf as *const ELF as *const u8).add(offset)}
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct ELF{
    identity : [u8;16],
    etype : Etype,
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SectionHeader{
    pub section_name : u32,
    stype : SType,
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

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SType {
    NULL = 0,
    ProgramBit = 1, // 包含程序定义的信息
    SymbolTable = 2,
    StringTable = 3,
    ReLa = 4,
    Hash = 5, // Symbol hash
    Dynamic = 6,
    Note = 7,
    NoBits = 8,
    Rel = 9,
    Reserved = 10,
    DynamicSymbol = 11,
    Env = 0x60000000,
    HiOS = 0x6FFFFFFF,
    LoProc = 0x70000000,
    HiProc = 0x7FFFFFFF,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ProgramHeader{
    ptype : PType,
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
        let t = {self.ptype};
        t == PType::Load
    }
    pub fn is_null(&self)->bool {
        let t = {self.ptype};
        t == PType::Null
    }

    pub fn va(&self)->usize {
        self.virtual_addr as usize
    }

    pub fn size(&self)->usize {
        self.segment_size_in_file as usize
    }

    pub fn offset(&self)->usize {
        self.offset_in_file as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u32)]
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

pub enum PFlag {
    Exec    = 1,
    Write   = 2,
    Read    = 4,
    MaskOS  = 0xff_0000,
    MaskProg= 0xff00_0000,
}

#[derive(Debug)]
#[repr(u16)]
pub enum Etype{
    None = 0,
    Rel = 1,
    Exec = 2,
    Dynamic = 3,
    Core = 4,
    LoPro = 0xff00,
    HiPro = 0xffff,
}

#[repr(u16)]
#[derive(Debug)]
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

use core::usize;
use alloc::string::String;
