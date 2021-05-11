//! # 窗口内容显示
//! 负责文件、文字显示
//! 
//! 2021年2月5日 zg

const FILE_SIZE : usize = 80;
const BACKGROUND : Pixel = Pixel::black();

#[derive(PartialEq, Clone, Copy)]
pub enum ContentType {
    Text,
    File,
}

pub enum ContentEvent {
    Exec(usize),
    Invalid,
    None,
}

pub struct Content {
    pub ctx : Grid,
    pub ctype : ContentType,
    pub directory : Directory,
    pub task_id : Option<usize>,
    pub idx : usize,
}

impl Content {
    pub fn new(x: usize, y: usize, width: usize, height: usize, ctype: ContentType,
            task_id:Option<usize>)->Self {
        let grid;
        let dir = enter("0/".to_string()).unwrap();
        match ctype {
            ContentType::Text => {
                let color = Pixel::black();
                grid = Grid::solid_color(x, y, width, height, FONT_WIDTH, FONT_HEIGHT, color);
            }
            ContentType::File => {
                let color = Pixel::white();
                grid = Grid::solid_color(x, y, width, height, FILE_SIZE, FILE_SIZE, color);
            }
        }
        
        let mut rt = Self {
            ctx : grid,
            ctype,
            directory : dir,
            task_id,
            idx : 0,
        };
        if ctype == ContentType::File {
            rt.refresh();
        }
        rt
    }

    pub fn scroll(&mut self, offset : isize) {
        self.ctx.scroll(offset);
    }

    pub fn draw(&mut self) {
        self.ctx.draw_override();
    }

    pub fn translate(&mut self, vec : Vector) {
        self.ctx.translate(vec);
    }

    pub fn get_key(&mut self, c : char) {
        // self.write_char(c, FONT_COLOR);
        if let Some(id) = self.task_id {
            println!("push {}", c);
            push_task_in(id, c);
        }
        println!("after get key");
    }

    pub fn do_mouse_event(&mut self, event : MouseEvent)->ContentEvent {
        match event.etype {
            super::mouse::MouseEventType::LeftDoubleClick => {
                if self.ctype == ContentType::Text {
                    return ContentEvent::None;
                }
                if let super::mouse::MouseEventInfo::Point(point) = event.info {
                    if let Some(idx) = self.detect(point) {
                        if idx < self.directory.item.len() {
                            let dir = self.directory.item.get(idx).unwrap();
                            if dir.is_dir() {
                                let p = self.directory.device_id.to_string()
                                    + &self.directory.path[..];
                                self.directory = enter(p + "/" + &dir.name[..]).unwrap();
                                self.refresh();
                            }
                            else {
                                let path = self.directory.device_id.to_string()
                                    + &self.directory.path[..] + "/"
                                    + &self.directory.item.get(idx).unwrap().name[..];
                                println!("path {}", path);
                                let id = exec(path);
                                if id > 0 {
                                    return ContentEvent::Exec(id)
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        return ContentEvent::None
    }

    pub fn run(&mut self)->ContentEvent {
        if let Some(task_id) = self.task_id {
            let mut s = String::new();
            while let Some(c) = pop_task_out(task_id) {
                s.push(c);
            }
            if s.len() > 0 {
                self.write_string(&s, Pixel::white());
                return ContentEvent::Invalid;
            }
        }
        return ContentEvent::None;
    }

    pub fn refresh(&mut self) {
        self.idx = 0;
        self.ctx.fill(Pixel::white());
        let dir = &mut self.directory;
        let b = TexBlock::new(FILE_SIZE, FILE_SIZE, 0);
        for file in dir.item.iter() {
            let image;
            if file.is_file() {
                image = request("0/img/file_black.bmp".to_string(), 60, 60).unwrap();
            }
            else {
                image = request("0/img/folder2.bmp".to_string(), 60, 60).unwrap();
            }
            b.fill_image(10, 0, image);
            let name = file.name.trim();
            let mut x = (FILE_SIZE - min(FILE_SIZE, name.len() * FONT_WIDTH)) / 2;
            for (_, c) in name.bytes().enumerate() {
                b.fill_font(x, 60, c as char, FONT_HEIGHT, FONT_WIDTH, Pixel::black(), Pixel::white());
                x += FONT_WIDTH;
            }
            self.ctx.from_block(self.idx, &b, true);
            self.idx += 1;
            b.fill(Pixel::white());
        }
    }

    pub fn detect(&self, point : Position)->Option<usize> {
        self.ctx.detect(point)
    }

    pub fn return_dir(&mut self) {
        if let Some(idx) = self.directory.path.find("/") {
            let (path, _) = self.directory.path.split_at(idx);
            let sys = get_system(self.directory.device_id).unwrap();
            self.directory = sys.enter(path.to_string()).unwrap();
            self.refresh();
        }
    }
}

impl Content {
    fn write_char(&mut self, c : char, color : Pixel) {
        if c == '\n' || c == '\r' {
            self.idx = self.idx + self.ctx.line_num - self.idx % self.ctx.line_num;
        }
        else {
            self.ctx.fill_font(self.idx, c, color, BACKGROUND);
            self.idx += 1;
        }
    }

    fn write_string(&mut self, s : &String, color : Pixel) {
        for c in s.bytes() {
            self.write_char(c as char, color);
        }
    }
}


use core::cmp::min;

use crate::{filesystem::{get_system, io::enter, pop_task_out, push_task_in, request}, graphic::canvas::texblock::TexBlock, libs::{shape::Position, syscall::exec}};
use alloc::prelude::v1::*;
use tisu_fs::{Directory, SystemOp};
use crate::{graphic::canvas::{grid::Grid}, libs::{font::{FONT_HEIGHT, FONT_WIDTH}, graphic::Pixel, shape::{Vector}}};

use super::mouse::MouseEvent;

