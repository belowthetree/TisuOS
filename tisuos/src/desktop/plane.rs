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
        let mut file = File::open(&"0/img/mac.bmp".to_string()).unwrap();
        file.open_flag(OpenFlag::Read.val()).unwrap();
        let mut image = generate_image(file).unwrap();
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
            if input != 0{
                let key = Key::from(input);
                // println!("press {:?} {}", key, input);
                match key {
                    Key::MouseLeft | Key::MouseRight | Key::MouseMid => {
                        self.mouse.get_key_down(key);
                        self.focus_window(Position::from_scale_point(get_mouse_position()));
                        self.invalid.set_true();
                    }
                    _ => {
                        self.keyboard.get_key_down(key);
                    }
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
            if input != 0{
                let key = Key::from(input);
                // println!("release {:?} {}", key, input);
                match key {
                    Key::MouseLeft | Key::MouseRight | Key::MouseMid => {
                        self.mouse.get_key_up(key);
                    }
                    _ => {
                        self.keyboard.get_key_release(key);
                    }
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
                asm!("wfi"::::"volatile");
                self.get_input();
                self.run_event();
                self.draw();
            }
        }
    }
}


use crate::{filesystem::{filetree::file::{File, OpenFlag}, image::bmp::generate_image},
    graphic::{colorblock::ColorBlock}, libs::shape::Position, sync::{Bool},
    virtio::{config::{HEIGHT, WIDTH}, device::invalid, input::{input_buffer::{get_key_press, get_key_release,
        get_mouse_position, get_scroll}, keyboard::Key}}};
use alloc::{prelude::v1::*};
use super::{content::ContentType, dock::{Dock, DockFuncType}, keyboard::{Keyboard}, mouse::{Mouse}, window::{Window}};
// use crate::uart;

