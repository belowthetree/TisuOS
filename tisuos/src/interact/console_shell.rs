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
        Self {
            directory : None,
            buffer : String::new(),
        }
    }

    pub fn make_command(&mut self) {
        let mut c = self.get_input();
        if c.is_none() {
            return;
        }
        
        while c.is_some() {
            output(c.unwrap() as u8);
            match c.unwrap() {
                '\n' | '\r' => {
                    self.do_command(&self.buffer.clone());
                    self.buffer.clear();
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
            c = self.get_input();
        }
    }

    pub fn do_command(&mut self, cmd : &String) {
        let s : Vec<&str> = cmd.split(' ').collect();
        if s.len() == 2{
            match s[0] {
                "draw" => {
                    if gpu_support() {
                        if let Some(dir) = &self.directory {
                            let sys = get_system(dir.device_id).unwrap();
                            let path = dir.path.clone() + "/" + s[1];
                            let file = sys.open(path.clone(), FileFlag::Read).unwrap().clone();
                            let data = Block::<u8>::new(file.size);
                            sys.read(file.id, data.to_array(0, file.size)).unwrap();
                            let mut img = BMP::decode(
                                data.to_array(0, file.size)).unwrap();
                            img.resize(100, 100);
                            let ptr = img.data.as_mut_slice();
                            let ptr = ptr as *mut [Pixel] as *mut Pixel;
                            println!("draw {}, w {} h {}", path, img.width, img.height);
                            draw_rect_override(0, Rect {
                                x1: 0,
                                y1: 0,
                                x2: img.width as u32,
                                y2: img.height as u32,
                            }, ptr);
                            invalid();
                        }
                    }
                    else {
                        println!("no graphic");
                    }
                }
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
                        let sys = get_system(dir.device_id).unwrap();
                        let file = 
                            sys.open(s[1].to_string(), FileFlag::Read).unwrap().clone();
                        let data = Block::<u8>::new(file.size);
                        sys.read(file.id, data.to_array(0, file.size)).unwrap();
                        let elf = data.type_as::<ELF>();
                        println!("{:x?}", elf);
                    }
                }
                "exec" => {
                    if let Some(dir) = &self.directory {
                        println!("exec {}", dir.path.clone() + s[1]);
                        exec("0".to_string() + &dir.path[..] + s[1]);
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
                        print!("directory: ");
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::Directory {
                                print!("{} ", item.name);
                            }
                        }
                        println!();
                        print!("file: ");
                        for item in tree.item.iter() {
                            if item.itype == DirItemType::File {
                                print!("{} ", item.name);
                            }
                        }
                        println!();
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
        pop()
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

fn output(c : u8){
    match c {
        10 | 13 => { println!(); }
        127 => { 
            print!("{} {}", 8 as char, 8 as char);
        }
        _ => { print!("{}", c as char); }
    }
}

use super::console_input::pop;
use alloc::prelude::v1::*;
use fs_format::BMP;
use tisu_driver::{Pixel, Rect};
use tisu_fs::{DirItemType, Directory, FileFlag};
use crate::{filesystem::{elf::ELF, get_system}, libs::{str::convert_to_usize, syscall::{exec, list_thread}}, memory::block::Block, virtio::device::{draw_rect_override, gpu_support, invalid}};
