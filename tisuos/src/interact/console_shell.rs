//! # 命令行交互壳
//! 接收 uart 输入进行交互，无图形界面下使用
//! 
//! 2021年1月25日 zg


/// ## 维护一个循环队列
pub struct ConsoleShell {
    directory : Option<Directory>,
    buffer : String,
}

impl ConsoleShell {
    pub fn new()->Self {
        println!("\ntype help to get some infomation\n");
        print!(">>");
        Self {
            directory : None,
            buffer : String::new(),
        }
    }

    /// 处理输入形成命令字符串
    pub fn make_command(&mut self) {
        while let Some(c) = self.get_input() {
            output(c as u8);
            match c {
                '\n' | '\r' => {
                    self.do_command(&self.buffer.clone());
                    self.buffer.clear();
                    output('>' as u8);
                    output('>' as u8);
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
                        println!("before enter {}", s[1]);
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
                        println!("get file id {}", id);
                        let path = dir.device_id.clone().to_string() + &path[..];
                        let addr = file_info(path) as *const FileInfo;
                        let info = unsafe {(*addr).clone()};
                        println!("{:x}\n{:?}", addr as usize, info);
                        let data = Block::<u8>::new(info.size);
                        let len = read(info.id, data.to_array(0, info.size));
                        let elf = data.type_as::<ELF>();
                        println!("read len {}\n{:x?}", len, elf);
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
                        println!("path {} size {} {:x}", path, file.size, file.start_idx);
                        let data = Block::<char>::new(file.size);
                        sys.read(file.id, data.to_array(0, file.size)).unwrap();
                        for c in data.array::<u8>(0, file.size) {
                            print!("{}", *c as char);
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
                "readdir" => {
                    let ptr = directory_info("0/".to_string());
                    let info = unsafe {&mut *(ptr as *mut filesystem::DirectoryInfo)};
                    println!("get info {:?}", info);
                    for i in 0..info.file_num {
                        let offset = i * 15 * size_of::<char>() + size_of::<filesystem::DirectoryInfo>();
                        println!("{}", from_ptr((ptr + offset) as *const char));
                    }
                }
                "help" => {
                    println!("
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
                        println!("{}", dir.path);
                    }
                    else {
                        println!("not in any filesystem, use cddisk to enter fs");
                    }
                }
                "ls" => {
                    if let Some(tree) = &self.directory{
                        output_str(&"directory: ".to_string());
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::Directory {
                                output_str(&(item.name.clone() + " "));
                            }
                        }
                        output_str(&"\nfile: ".to_string());
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::File {
                                output_str(&(item.name.clone() + " "));
                            }
                        }
                        output('\n' as u8);
                    }
                }
                "lsdisk" => {
                }
                "lsp" => {
                    list_thread();
                }
                "lsm" => {
                }
                "testfat" => {
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

fn output_str(s : &String) {
    for c in s.bytes() {
        output(c);
    }
}

fn output(c : u8){
    match c {
        10 | 13 => push_output('\n'),
        127 => { 
            push_output(8 as char);
            push_output(' ');
            push_output(8 as char);
        }
        _ => { push_output(c as char) }
    }
}

use core::mem::size_of;

use alloc::prelude::v1::*;
use tisu_driver::{Pixel};
use tisu_fs::{DirItemType, Directory, FileFlag, SystemOp};
use crate::{filesystem::{self, FileInfo, elf::ELF, get_system, pop_input, push_output}, libs::{str::{convert_to_usize, from_ptr}, syscall::{directory_info, draw_rect, exec, file_info, free, list_thread, open, read, wait}}, memory::block::Block};
