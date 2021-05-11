//! # 交互壳
//! 不直接与底层交互，接收输入产生事件
//! 
//! 2021年2月7日 zg

#![allow(dead_code)]

use crate::println;

pub enum ShellEvent {
    SwitchDirectory(String),
    List,
    SwitchDisk(usize),
    Exec(String),
}

/// ## 交互 Shell
/// 专门给图形界面的命令行用，功能类似，不过主要作为一个命令处理工具
pub struct InterShell{
    input_list : String,
}

impl InterShell {
    pub fn new()->Self{
        println!("inter shell new");
        Self{
            input_list : String::new(),
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
                }
                _ =>{}
            }
        }
        rt
    }
}


use alloc::prelude::v1::*;

use super::str::convert_to_usize;
