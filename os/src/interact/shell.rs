//! # Shell
//! 用于接受用户输入、与用户沟通，提供基本功能，用于测试内核
//! 默认生成一个 Shell，同时有一个函数作为独立进程循环触发每一个 Shell 进行更新
//! 2020年12月20日 zg

use alloc::{prelude::v1::*};
use bmp::generate_image;
/// ## Shell
/// file tree 一个 Shell 对应一个文件树，默认无目录，需要通过 cddisk 进入某一个磁盘的根目录
/// list 保存当前 Shell 获取的输入
pub struct Shell{
    file_tree : Option<FileTree>,
    input_list : String,
}
impl Shell {
    pub fn new() ->Self {
        Self{
            file_tree : None,
            input_list : String::new(),
        }
    }
    pub fn handle_cmd(&mut self, cmd : &String) {
        let s : Vec<&str> = cmd.split(' ').collect();
        if s.len() == 2{
            match s[0] {
                "cd" => {
                    self.enter_directory(s[1].to_string());
                }
                "cddisk" => {
                    self.file_tree = get_root(from_string(&s[1].to_string()));
                }
                "readelf" =>{
                    if let Some(tree) = &mut self.file_tree{
                        if let Some(mut file) = tree.get_file(s[1].to_string()){
                            if file.open(OpenFlag::Read.val()).is_ok(){
                                if let Some(buffer) = file.read(0, 512){
                                    let phr = unsafe {
                                        &*(buffer.addr as *const ProgramHeader)
                                    };
                                    phr.list();
                                }
                            }
                        } else {
                            println!("no file chain");
                        }
                    }
                }
                "exec" => {
                    if let Some(tree) = &mut self.file_tree{
                        if let Some(file) = tree.get_file(s[1].to_string()){
                            if let Some(res) = 
                                    load_elf(file){
                                exec(res.0, res.1);
                            }
                        }
                    }
                }
                "readimg" => {
                    if let Some(tree) = &mut self.file_tree{
                        if let Some(item) = tree.get_item(&s[1].to_string()){
                            if let Some(file) = item.get_file(){
                                let img = generate_image(file);
                                if let Some(img) = img{
                                    let mut style = Style::new_default();
                                    style.set_texture(img);
                                    style.resize(WIDTH as u32, HEIGHT as u32);
                                    style.draw_area();
                                }
                            }
                        }
                    }
                }
                "del" => {
                    if let Some(tree) = &self.file_tree{
                        delete_file(tree, &s[1].to_string());
                    }
                }
                "mkdir" => {
                    if let Some(tree) = &self.file_tree{
                        create_directory(tree, &s[1].to_string());
                    }
                }
                _ =>{}
            }
        }
        else if s.len() == 1 {
            match s[0] {
                "ls" => {
                    if let Some(tree) = &self.file_tree{
                        tree.list_file();
                        tree.list_dir();
                    }
                }
                "lsdisk" => {
                    if let Some(tree) = &self.file_tree{
                        print!("total size {}MB\t\t", tree.get_total_size() / 1024 / 1024);
                        println!("used size {}KB\t\t", tree.get_used_size() / 1024);
                    }
                }
                "lsp" => {
                    list_thread();
                }
                "testfat" => {
                    test();
                }
                _ =>{}
            }
        }
    }
    /// 由 Update 进程调用
    pub fn update(&mut self) {
        if self.input_list.len() > 0 {
            let c = &self.input_list[(self.input_list.len() - 1)..];
            if c == "\r" || c == "\n"{
                self.handle_cmd(&self.input_list[..self.input_list.len() - 1].to_string());
                
                self.input_list.clear();
            }
        }
    }
    pub fn handle(&mut self, c : char){
        output(c as u8);
        if c as u8 == 127 && self.input_list.len() > 0{
            self.input_list.pop();
        }
        else{
            self.input_list.push(c);
        }
    }
    fn enter_directory(&mut self, name : String){
        if let Some(tree) = &self.file_tree{
            if &name[..] == ".."{
                self.file_tree = tree.get_parent_directory();
            }
            else{
                println!("before enter");
                self.file_tree = tree.get_sub_directory(&name);
            }
        }
    }
}
/// 所有的 Shell 都保存在这
pub static mut SHELL : Option<Vec::<Shell>> = None;

pub fn init(){
    unsafe {
        SHELL = Some(Vec::<Shell>::new());
        if let Some(s) = &mut SHELL{
            s.push(Shell::new());
        }
        if let Some(s) = &mut SHELL{
            if let Some(i) = s.first(){
                if let Some(tree) = &i.file_tree{
                    tree.list_dir();
                }
            }
        }
    }
}

/// ## Update
/// 遍历所有 Shell 并调用它们的 update 函数
/// 会作为独立进程挂载，为了避免出现问题同时节省 CPU 消耗，触发中断前阻塞
pub fn update(){
    loop{
        unsafe{asm!("wfi"::::"volatile");}
        if let Some(s) = unsafe {&mut SHELL}{
            for f in s{
                f.update();
            }
        }
    }
}

fn from_string(s : &String) ->usize{
    let mut rt : usize = 0;
    for c in s.bytes(){
        rt = rt * 10 + (c- '0' as u8) as usize;
    }
    rt
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

use crate::{desktop::controll::style::{style::Style}, filesystem::{elf::{ProgramHeader, load_elf}, file::OpenFlag, operation, file_tree::FileTree}, libs::{str, syscall::{exec, list_thread}}, virtio::gpu_device::{HEIGHT, WIDTH}};
use operation::{create_directory, delete_file, get_root, test};
use crate::filesystem::image::bmp;
use crate::uart;