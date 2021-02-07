//! # 窗口内容显示
//! 负责文件、文字显示
//! 
//! 2021年2月5日 zg

const FILE_SIZE : usize = 80;
const FONT_COLOR : Pixel = Pixel::green();
const BACKGROUND : Pixel = Pixel::black();

#[derive(PartialEq, Clone, Copy)]
pub enum ContentType {
    Text,
    File,
}

pub struct Content {
    pub ctx : Grid,
    pub ctype : ContentType,
    pub directory : Directory,
    pub shell : InterShell,
    pub idx : usize,
}

impl Content {
    pub fn new(x : usize, y : usize, width : usize, height : usize, ctype : ContentType)->Self {
        let grid;
        let dir = Directory::open(&"0/".to_string()).unwrap();
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
            ctype : ctype,
            directory : dir,
            shell : InterShell::new(),
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
        self.write_char(c, FONT_COLOR);
        let rt = self.shell.input(c);
        if let Some(event) = rt {
            self.do_shell_event(event);
        }
    }
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
    fn do_shell_event(&mut self, event : ShellEvent) {
        match event {
            ShellEvent::SwitchDirectory(path) => {
                if path == ".." {
                    if let Some(dir) = self.directory.get_parent_directory() {
                        self.directory = dir;
                    }
                }
                else if let Some(dir) = self.directory.get_sub_directory(&path) {
                    self.directory = dir;
                }
            }
            ShellEvent::Output(s) => {
                self.write_string(&s, FONT_COLOR);
            }
            ShellEvent::List => {
                let mut items = Vec::<DirItem>::new();
                for file in self.directory.items.iter() {
                    let t = file.clone();
                    items.push(t);
                }
                for item in items {
                    let color;
                    if item.is_file() {
                        color = Pixel::green();
                    }
                    else {
                        color = Pixel::yellow();
                    }
                    self.write_string(&(item.name + " "), color);
                }
                self.write_char('\n', FONT_COLOR);
            }
            ShellEvent::SwitchDisk(_) => {}
            ShellEvent::Exec(_) => {}
        }
    }
    pub fn do_mouse_event(&mut self, event : MouseEvent) {
        match event.etype {
            super::mouse::MouseEventType::LeftDoubleClick => {
                if self.ctype == ContentType::Text {
                    return;
                }
                if let super::mouse::MouseEventInfo::Point(point) = event.info {
                    if let Some(idx) = self.detect(point) {
                        if idx < self.directory.items.len() {
                            let dir = self.directory.items.get(idx).unwrap();
                            if dir.is_dir() {
                                if dir.name == ".." {
                                    self.directory = self.directory.get_parent_directory().unwrap();
                                }
                                else {
                                    self.directory = self.directory.get_sub_directory(&dir.name).unwrap();
                                }
                                self.refresh();
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn refresh(&mut self) {
        self.idx = 0;
        self.ctx.fill(Pixel::white());
        let dir = &mut self.directory;
        let b = TexBlock::new(FILE_SIZE, FILE_SIZE, 0);
        for file in dir.items.iter() {
            let image;
            if file.is_file() {
                image = Image::request(&"0/img/file_black.bmp".to_string(), 60, 60).unwrap();
            }
            else {
                image = Image::request(&"0/img/folder_black.bmp".to_string(), 60, 60).unwrap();
            }
            b.fill_image(10, 0, &image);
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
        if let Some(dir) = self.directory.get_parent_directory() {
            self.directory = dir;
            if self.ctype == ContentType::File {
                self.refresh();
            }
        }
    }
}

use core::cmp::min;

use crate::{filesystem::{filetree::directory::{DirItem, Directory}, image::image::Image}, graphic::canvas::texblock::TexBlock, interact::intershell::{InterShell, ShellEvent}, libs::shape::Position};
use alloc::prelude::v1::*;
use crate::{graphic::canvas::{grid::Grid}, libs::{font::{FONT_HEIGHT, FONT_WIDTH}, graphic::Pixel, shape::{Vector}}};
use crate::uart;

use super::{keyboard::KeyboardEvent, mouse::MouseEvent};

