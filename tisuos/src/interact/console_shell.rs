//! # 命令行交互壳
//! 接收 uart 输入进行交互，无图形界面下使用
//! 
//! 2021年1月25日 zg


const BUFFER_SIZE : usize = 128;

/// ## 维护一个循环队列
pub struct ConsoleShell {
    read_idx : usize,
    write_idx : usize,
}

impl ConsoleShell {
    pub fn make_command(&mut self) {
        let mut c = self.get_input();
        let mut cmd = String::new();
        while c.is_some() {
            match c.unwrap() {
                '\n' | '\r' => {
                    self.do_command(&cmd);
                }
                _ => {
                    cmd.push(c.unwrap());
                }
            }
        }
    }

    pub fn do_command(&mut self, cmd : &String) {
        let s : Vec<&str> = cmd.split(' ').collect();
        if s.len() == 2{
            match s[0] {
                "cd" => {
                    // self.enter_directory(s[1].to_string());
                }
                "cddisk" => {
                    // self.filetree = get_root(from_string(&s[1].to_string()));
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
                    // if let Some(tree) = &mut self.filetree{
                    //     if let Some(file) = tree.get_file(s[1].to_string()){
                    //         if let Some(res) = 
                    //                 load_elf(file){
                    //             exec(res.0, res.1);
                    //         }
                    //     }
                    // }
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
                "ls" => {
                    // if let Some(tree) = &self.filetree{
                    //     tree.list_file();
                    //     tree.list_dir();
                    // }
                }
                "lsdisk" => {
                    // if let Some(tree) = &self.filetree{
                    //     print!("total size {}MB\t\t", tree.get_total_size() / 1024 / 1024);
                    //     println!("used size {}KB\t\t", tree.get_used_size() / 1024);
                    // }
                }
                "lsp" => {
                    // list_thread();
                }
                "testfat" => {
                    // test();
                }
                _ =>{}
            }
        }
    }
    /// ### 获取命令行输入
    fn get_input(&mut self)->Option<char> {
        get()
    }

}

use super::console_input::get;
use alloc::prelude::v1::*;
