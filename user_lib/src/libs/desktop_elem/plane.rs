//! # 桌面板
//! 作为桌面基础抽象，管理桌面系统所有元素
//! 
//! 2021年2月2日 zg


const WINDOW_WIDTH : usize = 300;
const WINDOW_HEIGHT : usize = 300;
const WINDOW_X : usize = (WIDTH - WINDOW_WIDTH) / 2;
const WINDOW_Y : usize = (HEIGHT - WINDOW_HEIGHT) / 2;

#[derive(Clone, Copy)]
pub enum PlaneEvent {
    None,
    WindowClose(usize),
    WindowSmall(usize),
    CreateFileWindow,
    CreateTerminal,
    WindowInvalid,
}

pub struct Plane {
    pub background : ColorBlock,
    pub dock : Dock,
    pub window : Vec<Window>,
    pub mouse : Mouse,
    pub keyboard : Keyboard,
    pub invalid : Bool,
}


impl Plane {
    pub fn new()->Self {
        let id = open("1/img/mac.bmp".to_string(), 1);
        let ptr = file_info("1/img/mac.bmp".to_string());
        let info = unsafe {&*(ptr as *const u8 as *const FileInfo)};
        let data = Block::<u8>::new(info.size);
        assert!(id > 0);
        read(id as usize, data.to_array(0, info.size));
        let mut image = BMP::decode(data.to_array(0, info.size)).unwrap();
        image.resize(WIDTH, HEIGHT);
        let mut rt = Self {
            background : ColorBlock::image(0, 0, &image),
            window : Vec::<Window>::new(),
            dock : Dock::new(),
            mouse : Mouse::new(),
            keyboard : Keyboard::new(),
            invalid : Bool::new(),
        };
        rt.create_window();
        rt.invalid.set_true();
        rt
    }
    pub fn draw(&mut self) {
        if !self.invalid.pop() {
            return;
        }
        self.background.draw_override();
        for window in self.window.iter_mut() {
            window.draw();
        }
        self.dock.draw();
        invalid();
    }
    fn create_window(&mut self) {
        let window = Window::new(WINDOW_X, WINDOW_Y, WINDOW_WIDTH, WINDOW_HEIGHT, ContentType::File);
        self.dock.add_func(DockFuncType::TriggerFileWindow(window.id));
        self.window.push(window);
        self.invalid.set_true();
    }
    fn create_terminal(&mut self) {
        let window = Window::new(WINDOW_X, WINDOW_Y, WINDOW_WIDTH, WINDOW_HEIGHT, ContentType::Text);
        self.dock.add_func(DockFuncType::TriggerTerminal(window.id));
        self.window.push(window);
        self.invalid.set_true();
    }
    fn do_event(&mut self, event : PlaneEvent) {
        match event {
            PlaneEvent::WindowInvalid => {
                self.invalid.set_true();
            }
            PlaneEvent::CreateTerminal => {
                self.create_terminal();
            }
            PlaneEvent::CreateFileWindow => {
                self.create_window();
            }
            PlaneEvent::WindowClose(id) => {
                self.remove_window(id);
            }
            PlaneEvent::WindowSmall(id) => {
                for window in self.window.iter_mut() {
                    if window.id == id {
                        window.trigger_hidden();
                        break;
                    }
                }
                self.invalid.set_true();
            }
            _ => {
            }
        }
    }
    fn remove_window(&mut self, id : usize) {
        for (idx, window) in self.window.iter_mut().enumerate() {
            if window.id == id {
                self.dock.remove_func(id);
                self.window.remove(idx);
                self.invalid.set_true();
                break;
            }
        }
    }
    fn get_key_down(&mut self) {
        loop {
            let input = get_key_press();
            if let Some(input) = input {
                let key = Key::from_code(input as usize);
                // println!("press {:?} {}", key, input);
                match key {
                    Ok(Key::MouseLeft | Key::MouseRight | Key::MouseMid) => {
                        self.mouse.get_key_down(key.unwrap());
                        self.focus_window(Position::from_scale_point(get_mouse_position()));
                        self.invalid.set_true();
                    }
                    Ok(key) => {
                        self.keyboard.get_key_down(key);
                    }
                    _ => {}
                }
            }
            else{
                break;
            }
        }
    }
    fn get_key_release(&mut self) {
        loop {
            let input = get_key_release();
            if let Some(input) = input {
                let key = Key::from_code(input as usize);
                // println!("release {:?} {}", key, input);
                match key {
                    Ok(Key::MouseLeft | Key::MouseRight | Key::MouseMid) => {
                        self.mouse.get_key_up(key.unwrap());
                    }
                    Ok(key) => {
                        self.keyboard.get_key_release(key);
                    }
                    _ => {}
                }
            }
            else{
                break;
            }
        }
    }
    fn get_mouse_scroll(&mut self) {
        loop {
            let input = get_scroll();
            if input != 0{
                self.mouse.scroll(input);
            }
            else{
                break;
            }
        }
    }
    fn get_input(&mut self){
        self.get_key_down();
        self.get_key_release();
        self.get_mouse_scroll();
        self.mouse.update_position();
        // loop{
        //     let e = self.mouse.get_event();
        //     // println!("{:?}", e);
        //     if e == MouseEvent::None{
        //         break;
        //     }
        //     self.cmd(e);
        // }
    }
    fn run_event (&mut self) {
        let mut v = Vec::<PlaneEvent>::new();
        if let Some(window) = self.window.last_mut() {
            let mut event = self.keyboard.pop_event();
            while event.is_some() {
                let e = window.do_keyboard_event(event.unwrap());
                event = self.keyboard.pop_event();
                v.push(e);
            }
            let mut event = self.mouse.pop_event();
            while event.is_some() {
                let mut e;
                if window.focus(self.mouse.get_current_position()) {
                    e = window.do_mouse_event(event.unwrap());
                    v.push(e);
                }
                e = self.dock.do_mouse_event(event.unwrap());
                event = self.mouse.pop_event();
                v.push(e);
            }
        }
        else {
            let mut event = self.mouse.pop_event();
            while event.is_some() {
                let e = self.dock.do_mouse_event(event.unwrap());
                event = self.mouse.pop_event();
                v.push(e);
            }
        }
        while v.len() > 0 {
            self.do_event(*v.last().unwrap());
            v.pop();
        }
    }
    fn focus_window(&mut self, point : Position) {
        for (idx, window) in self.window.iter_mut().rev().enumerate() {
            if window.focus(point) {
                let len = self.window.len();
                self.window.swap(len - idx - 1, len - 1);
                break;
            }
        }
    }
}


impl Plane {
    pub fn run(&mut self) {
        loop {
            unsafe {
                asm!("wfi");
                self.get_input();
                self.run_event();
                self.draw();
            }
        }
    }
}

use fs_format::{BMP, FileInfo};
use tisu_sync::Bool;
use alloc::prelude::v1::*;
use virtio_input_decoder::Key;
use crate::{HEIGHT, Position, WIDTH, file_info, libs::{graphic::{ColorBlock, invalid}, input::{get_key_press, get_key_release, get_mouse_position, get_scroll}, memory_block::Block}, open, read};

use super::{
    content::ContentType,
    dock::{
        Dock,
        DockFuncType
    },
    keyboard::Keyboard,
    mouse::Mouse,
    window::Window
};


