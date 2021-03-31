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
                _ => {
                    self.buffer.push(c.unwrap());
                }
            }
            c = self.get_input();
        }
    }

    pub fn do_command(&mut self, cmd : &String) {
        let s : Vec<&str> = cmd.split(' ').collect();
        if s.len() == 2{
            match s[0] {
                "cd" => {
                    self.enter_directory(s[1].to_string());
                }
                "cddisk" => {
                    self.directory = get_directory(convert_to_usize(&s[1].to_string()), 2);
                }
                "readelf" =>{
                    // if let Some(tree) = &mut self.filetree{
                    //     if let Some(mut file) = tree.get_file(s[1].to_string()){
                    //         if file.open(OpenFlag::Read.val()).is_ok(){
                    //             if let Some(buffer) = file.read(0, 512){
                    //                 let phr = unsafe {
                    //                     &*(buffer.addr as *const ProgramHeader)
                    //                 };
                    //                 phr.list();
                    //             }
                    //         }
                    //     } else {
                    //         println!("no file chain");
                    //     }
                    // }
                }
                "exec" => {
                    if let Some(dir) = &mut self.directory{
                        if let Some(file) = dir.get_file(s[1].to_string()){
                            load_elf(file)
                        }
                    }
                }
                "del" => {
                    // if let Some(tree) = &self.filetree{
                    //     delete_file(tree, &s[1].to_string());
                    // }
                }
                "mkdir" => {
                    // if let Some(tree) = &self.filetree{
                    //     create_directory(tree, &s[1].to_string());
                    // }
                }
                _ =>{}
            }
        }
        else if s.len() == 1 {
            match s[0] {
                "draw" => {
                    {
                        let color = Pixel::black();
                        let mut color = Grid::solid_color(0, 0, 100, 100, 20, 20, color);
                        let mut i = 0;
                        for c in 'a' as u8..='z' as u8 {
                            color.fill_font(i, c as char, Pixel::white(), Pixel::black());
                            i += 1;
                        }
                        color.scroll(20);
                        color.draw_override();
                        let mut file = File::open(&"0/img/mac.bmp".to_string()).unwrap();
                        println!("get file");
                        file.open_flag(OpenFlag::Read.val()).unwrap();
                        let mut img = generate_image(file).unwrap();
                        println!("get img");
                        img.resize(500, 300);
                        img.updown();
                        println!("resize");
                        let img = ColorBlock::image(0, 0, &img);
                        println!("get colorblock");
                        img.draw_override();
                        println!("after draw");
                        invalid();
                    }
                }
                "ls" => {
                    if let Some(tree) = &self.directory{
                        print!("directory: ");
                        for item in tree.items.iter() {
                            if item.is_dir() {
                                print!("{} ", item.name);
                            }
                        }
                        println!();
                        print!("file: ");
                        for item in tree.items.iter() {
                            if item.is_file() {
                                print!("{} ", item.name);
                            }
                        }
                        println!();
                    }
                }
                "lsdisk" => {
                    // if let Some(tree) = &self.filetree{
                    //     print!("total size {}MB\t\t", tree.get_total_size() / 1024 / 1024);
                    //     println!("used size {}KB\t\t", tree.get_used_size() / 1024);
                    // }
                }
                "lsp" => {
                    list_thread();
                }
                "lsm" => {
                }
                "testfat" => {
                    // test();
                }
                _ =>{}
            }
        }
    }
    
    fn enter_directory(&mut self, name : String){
        if let Some(tree) = &self.directory{
            if &name[..] == ".."{
                if let Some(dir) = tree.get_parent_directory() {
                    self.directory = Some(dir);
                }
            }
            else{
                if let Some(dir) = tree.get_sub_directory(&name) {
                    self.directory = Some(dir);
                }
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
            asm!("wfi"::::"volatile");
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
use crate::{filesystem::{elf::load_elf, filetree::{directory::{Directory, get_directory}, file::{File, OpenFlag}}, image::bmp::generate_image}, graphic::{canvas::grid::Grid, colorblock::ColorBlock}, libs::{graphic::Pixel,
        str::convert_to_usize, syscall::{list_thread}}, uart, virtio::device::invalid};
