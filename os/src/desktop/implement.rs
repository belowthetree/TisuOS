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
        let mut file = open(&"0/mac.bmp".to_string()).unwrap();
        if file.open(OpenFlag::Read.val()).is_ok(){
            let image = generate_image(file).unwrap();
            back.set_texture(image);
        }
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
        self.remove_window();
    }
    pub fn draw(&self){
        self.background.draw_area();
        for term in self.terminal.iter(){
            term.draw();
        }
        for window in self.window.iter(){
            window.draw();
        }
        refresh();
    }
    pub fn setup_background(&mut self, path : &String){
        let file = open(path).unwrap();
        let image = generate_image(file).unwrap();
        self.background.set_texture(image);
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
            // println!("{:?}", e);
        }
    }
    pub fn cmd(&mut self, event : MouseEvent){
        match event {
            MouseEvent::RightClick => {
                let p = self.mouse.cur_pos.clone();
                self.create_terminal((p.x * WIDTH as f32) as u32, (p.y * HEIGHT as f32) as u32);
            }
            MouseEvent::LeftClick => {
                if let Some(_) = self.select_window(self.mouse.pre_pos){
                }
            }
            MouseEvent::Drag => {
                let dir = self.mouse.get_move_dir();
                if let Some(term) = self.select_window(self.mouse.pre_pos){
                    term.translate(dir.0, dir.1);
                    self.valid = false;
                }
            }
            _ => {}
        }
    }
    pub fn select_window<'a>(&'a mut self, point : Point)->Option<&'a mut Terminal>{
        let point = Position::from_point(point);
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
    pub fn remove_window(&mut self){
        for (idx, t) in self.terminal.iter().enumerate() {
            if t.is_close() {
                println!("get close");
                self.terminal.remove(idx);
                self.valid = false;
                break;
            }
        }
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

static mut TERMINAL_ID : usize = 0;

impl Terminal {
    pub fn new(x : u32, y : u32, width : u32, height : u32)->Self{
        let id;
        unsafe {
            id = TERMINAL_ID;
            TERMINAL_ID += 1;
        }
        let rt = Self{
            x : x,
            y : y,
            width : width,
            height : height,
            depth : 0,
            head_bar : HeadBar::new(x, y, width, HEADBAR_HEIGHT),
            text : TextContent::new(x, y, width, height),
            id : id,
        };
        rt
    }
    pub fn draw(&self){
        self.head_bar.draw();
        self.text.draw();
    }
    pub fn is_close(&self)->bool{
        self.head_bar.close_button.click
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

    fn detect(&mut self, point : Position)->bool {
        let x = point.x;
        let y = point.y;
        let rt = self.x <= x && self.y <= y && self.x + self.width >= x && self.y + self.height >= y;
        if rt {
            self.head_bar.detect(point);
        }
        rt
    }

    fn translate(&mut self, x : i32, y : i32) {
        let mut x = x;
        let mut y = y;
        let mut xx = self.x as i32 + x;
        let mut yy = self.y as i32 + y;
        if xx < 0{
            xx = 0;
            x = - (self.x as i32);
        }
        if yy < 0{
            yy = 0;
            y = - (self.y as i32);
        }
        self.x = xx as u32;
        self.y = yy as u32;
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

    fn detect(&mut self, point : Position)->bool {
        self.content.detect(point)
    }

    fn translate(&mut self, x : i32, y : i32) {
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

    fn detect(&mut self, point : Position)->bool {
        let x = point.x;
        let y = point.y;
        let rt = self.x <= x && self.y <= y && self.x + self.width >= x && self.y + self.height >= y;
        if rt {
            self.close_button.detect(point);
        }
        rt
    }

    fn translate(&mut self, x : i32, y : i32) {
        let mut xx = self.x as i32 + x;
        let mut yy = self.y as i32 + y;
        if xx < 0{
            xx = 0;
        }
        if yy < 0{
            yy = 0;
        }
        self.x = xx as u32;
        self.y = yy as u32;
        self.background.translate(x, y);
        self.close_button.translate(x, y);
    }
}

use crate::{filesystem::{file::OpenFlag, image::bmp::generate_image, interface::open}, virtio::{gpu_device::{HEIGHT, Pixel, WIDTH, refresh}, input::{input_buffer::{Point, get_key_press, get_key_release}, keyboard::Key}}};
use crate::uart;
use super::{controll::{button::{BUTTON_WIDTH, Button}, style::style::{ColorStyle, Style}},
    desktop::{Desktop, Dock, HeadBar, Position, Terminal, TextContent, Window, register_desktop}, desktop_trait::{Transform}, mouse::{Mouse, MouseEvent}};
use alloc::{prelude::v1::*};
