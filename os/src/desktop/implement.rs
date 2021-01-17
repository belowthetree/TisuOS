//! # desktop 文件函数实现
//! 2020年12月30日 zg

#![allow(dead_code)]
/// ## Desktop
/// 

const TERMINAL_WIDTH : u32 = 300;
const TERMINAL_HEIGHT : u32 = 300;

impl Desktop {
    pub fn new(){
        let mut back = Style::new_default();
        back.resize(WIDTH as u32, HEIGHT as u32);
        let rt = Self{
            valid : false,
            mouse : Mouse::new(),
            background : back,
            dock : Dock::new(),
            terminal : Vec::<Terminal>::new(),
            window : Vec::<Window>::new(),
            cnt : 0,
        };
        register_desktop(rt);
    }
    pub fn run(&mut self){
        self.get_input();
        if !self.valid {
            self.draw();
            self.valid = true;
        }
    }
    pub fn draw(&self){
        self.background.draw_area();
        for term in self.terminal.iter(){
            term.draw();
        }
        for window in self.window.iter(){
            window.draw();
        }
    }
    pub fn create_terminal(&mut self, x : u32, y : u32){
        self.valid = false;
        let terminal = Terminal::new(x, y, TERMINAL_WIDTH, TERMINAL_HEIGHT);
        self.terminal.push(terminal);
    }
    pub fn get_input(&mut self){
        loop {
            let input = get_key_press();
            if input != 0{
                let key = Key::from(input);
                // println!("press {:?} {}", key, input);
                match key {
                    Key::MouseLeft | Key::MouseRight | Key::MouseMid => {
                        self.mouse.get_key_down(key);
                    }
                    _ => {
                    }
                }
            }
            else{
                break;
            }
        }
        
        loop {
            let input = get_key_release();
            if input != 0{
                let key = Key::from(input);
                // println!("release {:?} {}", key, input);
                match key {
                    Key::MouseLeft | Key::MouseRight | Key::MouseMid => {
                        self.mouse.get_key_up(key);
                    }
                    _ => {
                    }
                }
            }
            else{
                break;
            }
        }
        self.mouse.update_position();
        loop{
            let e = self.mouse.get_event();
            if e == MouseEvent::None{
                break;
            }
            self.cmd(e);
            println!("{:?}", e);
        }
    }
    pub fn cmd(&mut self, event : MouseEvent){
        match event {
            MouseEvent::RightClick => {
                let p = self.mouse.cur_pos.clone();
                self.create_terminal((p.x * WIDTH as f32) as u32, (p.y * HEIGHT as f32) as u32);
            }
            MouseEvent::LeftClick => {
                if let Some(term) = self.select_window(self.mouse.pre_pos){
                    println!("get term");
                }
            }
            MouseEvent::Drag => {
                let dir = self.mouse.get_move_dir();
                if let Some(term) = self.select_window(self.mouse.pre_pos){
                    println!("get term");
                    let x = (term.x as i32 + dir.0) as u32;
                    let y = (term.y as i32 + dir.1) as u32;
                    println!("{} {} {} {}", term.x, term.y, x, y);
                    term.translate(dir.0, dir.1);
                    self.valid = false;
                }
            }
            _ => {}
        }
    }
    pub fn select_window<'a>(&'a mut self, point : Point)->Option<&'a mut Terminal>{
        if self.terminal.len() > 0 {
            for term in self.terminal.iter_mut(){
                if term.detect(point) {
                    return Some(term);
                }
            }
            None
        }
        else {
            None
        }
    }
    pub fn get_mouse_point(&self)->Point{
        self.mouse.cur_pos
    }
}

/// ## Dock
/// 

impl Dock{
    pub fn new()->Self{
        Self{
        }
    }
}

/// ## Window
/// 

impl Window {
    pub fn new()->Self{
        Self{

        }
    }
    pub fn draw(&self){

    }
}

///
/// Terminal
///

impl Terminal {
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        let rt = Self{
            x : x,
            y : y,
            width : width,
            height : height,
            depth : 0,
            head_bar : HeadBar::new(x, y, width, HEADBAR_HEIGHT),
            text : TextContent::new(x, y, width, height)
        };
        rt
    }
    pub fn draw(&self){
        self.head_bar.draw();
        self.text.draw();
    }
}

impl Transform for Terminal {
    fn set_position(&mut self, x : u32, y : u32) {
        self.head_bar.set_position(x, y);
        self.text.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&self, point : Point)->bool {
        let x = (point.x * WIDTH as f32) as u32;
        let y = (point.y * HEIGHT as f32) as u32;
        self.x <= x && self.y <= y && self.x + self.width >= x && self.y + self.height >= y
    }

    fn translate(&mut self, x : i32, y : i32) {
        let mut x = self.x as i32 + x;
        let mut y = self.y as i32 + y;
        if x < 0{
            x = 0;
        }
        if y < 0{
            y = 0;
        }
        self.x = x as u32;
        self.y = y as u32;
        self.head_bar.translate(x, y);
        self.text.translate(x, y);
    }
}

/// ## TextContent

impl TextContent {
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        let color = ColorStyle::SolidColor;
        let mut content = Style::new(color, x, y + HEADBAR_HEIGHT, width, height - HEADBAR_HEIGHT);
        content.set_color(Pixel::green());
        Self{
            width : width,
            height : height,
            content : content
        }
    }
    pub fn draw(&self){
        self.content.draw_area();
    }
}

impl Transform for TextContent {
    fn set_position(&mut self, x : u32, y : u32) {
        self.content.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&self, point : Point)->bool {
        false
    }

    fn translate(&mut self, x : i32, y : i32) {
        // let mut x = self.x as i32 + x;
        // let mut y = self.y as i32 + y;
        // if x < 0{
        //     x = 0;
        // }
        // if y < 0{
        //     y = 0;
        // }
        self.content.translate(x, y);
    }
}

///
/// Headbar

const HEADBAR_HEIGHT : u32 = 20;

impl HeadBar {
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        let btn = Button::new(x + width - BUTTON_WIDTH, y, BUTTON_WIDTH, height);
        let mut back = Style::new(ColorStyle::SolidColor, x, y, width, height);
        back.set_color(Pixel::white());
        
        Self{
            background : back,
            close_button : btn,
            x : x,
            y : y,
            width : width,
            height : height,
            button_width : 30,
            close_button_offset : 470,
        }
    }
    pub fn new_default()->Self{
        let mut btn = Button::new_default();
        btn.set_position(470, 0);
        let mut back = Style::new(ColorStyle::SolidColor, 0, 0, 500, HEADBAR_HEIGHT);
        back.set_color(Pixel::white());
        Self{
            background : back,
            close_button : btn,
            x : 0,
            y : 0,
            width : 500,
            height : 20,
            button_width : 30,
            close_button_offset : 470,
        }
    }
    pub fn draw(&self){
        self.background.draw_area();
        self.close_button.draw();
    }
}

impl Transform for HeadBar {
    fn set_position(&mut self, x : u32, y : u32) {
        self.x = x;
        self.y = y;
        self.background.set_position(x, y);
        self.close_button.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&self, point : Point)->bool {
        false
    }

    fn translate(&mut self, x : i32, y : i32) {
        let mut x = self.x as i32 + x;
        let mut y = self.y as i32 + y;
        if x < 0{
            x = 0;
        }
        if y < 0{
            y = 0;
        }
        self.x = x as u32;
        self.y = y as u32;
        self.background.translate(x, y);
        self.close_button.translate(x, y);
    }
}

use crate::{virtio::{gpu_device::{HEIGHT, Pixel, WIDTH}, input::{input_buffer::{KEY_RELEASE_CUR_IDX, KEY_RELEASE_GET_IDX, Point, get_key_press, get_key_release}, keyboard::Key}}};
use crate::uart;
use super::{controll::{button::{BUTTON_WIDTH, Button}, style::style::{ColorStyle, Style}}, desktop::{Desktop, Dock, HeadBar, Terminal, TextContent, Window, register_desktop}, desktop_trait::Transform, mouse::{Mouse, MouseEvent}};
use alloc::{prelude::v1::*};
