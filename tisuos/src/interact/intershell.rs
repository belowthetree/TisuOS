//! # 交互壳
//! 不直接与底层交互，接收输入产生事件
//! 
//! 2021年2月7日 zg


pub enum ShellEvent {
    SwitchDirectory(String),
    Output(String),
    List,
    SwitchDisk(usize),
    Exec(String),
}

/// ## 交互 Shell
/// 专门给图形界面的命令行用，功能类似，不过主要做一个命令处理工具
pub struct InterShell{
    input_list : String,
    output_list : Vec<ColorChar>,
}

impl InterShell {
    pub fn new()->Self{
        println!("inter shell new");
        Self{
            input_list : String::new(),
            output_list : Vec::<ColorChar>::new(),
        }
    }
    pub fn input(&mut self, c : char)->Option<ShellEvent> {
        if c as u8 == 127 && self.input_list.len() > 0{
            self.input_list.pop();
        }
        else{
            self.input_list.push(c);
        }
        let mut e = None;
        if self.input_list.len() > 0 {
            let c = &self.input_list[(self.input_list.len() - 1)..];
            if c == "\r" || c == "\n"{
                e = self.cmd(&self.input_list[..self.input_list.len() - 1].to_string());
                
                self.input_list.clear();
            }
        }
        e
    }
    pub fn cmd(&mut self, cmd : &String)->Option<ShellEvent> {
        let s : Vec<&str> = cmd.split(' ').collect();
        let mut rt = None;
        if s.len() == 2{
            match s[0] {
                "cd" => {
                    rt = Some(ShellEvent::SwitchDirectory(s[1].to_string()));
                }
                "cddisk" => {
                    rt = Some(ShellEvent::SwitchDisk(convert_to_usize(&s[1].to_string())));
                }
                "readelf" =>{
                }
                "exec" => {
                    rt = Some(ShellEvent::Exec(s[1].to_string()));
                }
                "del" => {
                }
                "mkdir" => {
                }
                _ =>{}
            }
        }
        else if s.len() == 1 {
            match s[0] {
                "ls" => {
                    rt = Some(ShellEvent::List);
                }
                "lsdisk" => {
                }
                "lsp" => {
                    list_thread();
                }
                _ =>{}
            }
        }
        rt
    }
    pub fn add_output(&mut self, s : &mut Vec<ColorChar>){
        self.output_list.append(s);
    }
    pub fn output(&mut self, s : &String, color : Pixel){
        for c in s.bytes() {
            self.output_list.push(ColorChar{color : color, c : c as char});
        }
    }
    pub fn pop_output(&mut self)->Vec<ColorChar>{
        let rt = self.output_list.clone();
        self.output_list.clear();
        rt
    }
    fn make_color(&self, cs : &mut Vec<ColorChar>, s : &String, color : Pixel) {
        for c in s.bytes() {
            cs.push(ColorChar{c:c as char,color:color});
        }
    }
}


use alloc::prelude::v1::*;
use crate::{libs::{graphic::{ColorChar, Pixel}, str::convert_to_usize, syscall::list_thread}, uart};
