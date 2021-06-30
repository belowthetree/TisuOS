//! # 命令行交互壳
//! 接收 uart 输入进行交互，无图形界面下使用
//! 
//! 2021年1月25日 zg

// const TEST_FAIL         : u32 = 0x3333;
const TEST_PASS         : u32 = 0x5555;
// const TEST_RESET        : u32 = 0x7777;

/// ## 维护一个循环队列
pub struct ConsoleShell {
    directory : Option<Directory>,
    buffer : String,
}

impl ConsoleShell {
    pub fn new()->Self {
        console!("\ntype help to get some infomation\n");
        console!(">>");
        Self {
            directory : None,
            buffer : String::new(),
        }
    }

    /// 处理输入形成命令字符串
    pub fn make_command(&mut self) {
        while let Some(c) = self.get_input() {
            console!("{}", c);
            match c {
                '\n' | '\r' => {
                    self.do_command(&self.buffer.clone());
                    self.buffer.clear();
                    console!("\n>>");
                }
                c => {
                    if c as u8 == 127 {
                        self.buffer.pop();
                    }
                    else {
                        self.buffer.push(c);
                    }
                }
            }
        }
    }

    /// 命令执行部分
    /// 大部分是测试用例
    pub fn do_command(&mut self, cmd : &String) {
        let s : Vec<&str> = cmd.split(' ').collect();
        if s.len() == 2{
            match s[0] {
                "cd" => {
                    if let Some(dir) = &self.directory {
                        let sys = get_system(dir.device_id).unwrap();
                        // if s[1] == ".." && s[1].contains("/") {
                        //     let path : Vec<&str> = dir.path.split("/").collect();
                        // }
                        console!("before enter {}", s[1]);
                        self.directory = Some(sys.enter(s[1].to_string()).unwrap());
                    }
                }
                "cddisk" => {
                    let sys = 
                        get_system(convert_to_usize(&s[1].to_string())).unwrap();
                    self.directory = Some(sys.enter(String::from("")).unwrap());
                }
                "readelf" =>{
                    if let Some(dir) = &self.directory {
                        let path : String = dir.path.clone() + &s[1];
                        let id = open(
                            dir.device_id.to_string() + &path[..], FileFlag::Read.val());
                        console!("get file id {}", id);
                        let path = dir.device_id.clone().to_string() + &path[..];
                        let addr = file_info(path) as *const FileInfo;
                        let info = unsafe {(*addr).clone()};
                        console!("{:x}\n{:?}", addr as usize, info);
                        let data = Block::<u8>::new(info.size);
                        let len = read(info.id, data.to_array(0, info.size));
                        let elf = data.type_as::<ELF>();
                        console!("read len {}\n{:x?}", len, elf);
                        free(addr as usize);
                    }
                }
                "exec" => {
                    if let Some(dir) = &self.directory {
                        let id = exec(
                            dir.device_id.to_string() + &dir.path[..] + s[1]);
                        wait(id);
                    }
                }
                "cat" => {
                    if let Some(dir) = &self.directory {
                        let sys = get_system(dir.device_id).unwrap();
                        let path = dir.path.clone() + "/" + s[1];
                        let file = sys.open(path.clone(), FileFlag::Read).unwrap().clone();
                        console!("path {} size {} {:x}", path, file.size, file.start_idx);
                        let data = Block::<char>::new(file.size);
                        sys.read(file.id, data.to_array(0, file.size)).unwrap();
                        for c in data.array::<u8>(0, file.size) {
                            console!("{}", *c as char);
                        }
                    }
                }
                "mkdir" => {
                }
                _ =>{}
            }
        }
        else if s.len() == 1 {
            match s[0] {
                "shutdown" => {
                    unsafe {
                        console!();
                        const VIRT_TEST: *mut u32 = 0x10_0000 as *mut u32;
                        VIRT_TEST.write_volatile(TEST_PASS);
                    }
                }
                "time" => {
                    console!("\n{:?}", crate::rtc::Time::read());
                }
                "readdir" => {
                    let ptr = directory_info("0/".to_string());
                    let info = unsafe {&mut *(ptr as *mut filesystem::DirectoryInfo)};
                    console!("get info {:?}", info);
                    for i in 0..info.file_num {
                        let offset = i * 15 * size_of::<char>() + size_of::<filesystem::DirectoryInfo>();
                        console!("{}", from_ptr((ptr + offset) as *const char));
                    }
                }
                "help" => {
                    console!("
# use cddisk # to enter disk first
# use readelf to read elf infomation
# use exec to execute a binary
# use cat to watch one files content
# use ls to see current directory's infomation
                    ");
                }
                "draw" => {
                    let data = Block::<Pixel>::new(100);
                    let buffer = data.array(0, 100);
                    for i in 0..buffer.len() {
                        buffer[i] = Pixel::red();
                    }
                    draw_rect(0, 0, 10, 10, buffer);
                }
                "curdir" => {
                    if let Some(dir) = &self.directory {
                        console!("{}", dir.path);
                    }
                    else {
                        console!("not in any filesystem, use cddisk to enter fs");
                    }
                }
                "ls" => {
                    if let Some(tree) = &self.directory{
                        console!("directory: ");
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::Directory {
                                console!("{} ", item.name.clone());
                            }
                        }
                        console!("file: ");
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::File {
                                console!("{} ", item.name.clone());
                            }
                        }
                    }
                }
                "lsdisk" => {
                }
                "lsp" => {
                    list_thread();
                }
                "lsm" => {
                }
                "send" => {
                    console!("send {:?}", Ip::new());
                    get_device().send();
                }
                _ =>{}
            }
        }
    }

    /// ### 获取命令行输入
    fn get_input(&mut self)->Option<char> {
        pop_input()
    }

}

pub fn run() {
    let mut console = ConsoleShell::new();
    loop {
        unsafe {
            asm!("wfi");
        }
        console.make_command();
    }
}

use core::mem::size_of;

use alloc::prelude::v1::*;
use tisu_driver::{Pixel};
use tisu_fs::{DirItemType, Directory, FileFlag, SystemOp};
use crate::{console, filesystem::{self, FileInfo, elf::ELF, get_system, pop_input}, libs::{str::{convert_to_usize, from_ptr}, syscall::{directory_info, draw_rect, exec, file_info, free, list_thread, open, read, wait}}, memory::block::Block, virtio::{device::get_device, ip::Ip}};
