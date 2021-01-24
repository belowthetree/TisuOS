//! # desktop 功能实现
//! 
//! 2020年12月30日 zg

#![allow(dead_code)]
/// ## 桌面系统
/// 统一管理所有窗口、Dock

const WINDOW_WIDTH : usize = 300;
const WINDOW_HEIGHT : usize = 300;

impl Desktop {
    /// ### 新建桌面并向全局注册
    /// 未来考虑多桌面
    pub fn new(){
        let mut back = Style::new_default();
        let mut file = open(&"0/img/mac.bmp".to_string()).unwrap();
        if file.open(OpenFlag::Read.val()).is_ok(){
            let image = generate_image(file).unwrap();
            back.set_texture(image);
        }
        back.resize(WIDTH as usize, HEIGHT as usize);
        let mut rt = Self{
            valid : false,
            mouse : Mouse::new(),
            background : back,
            dock : Dock::new(),
            window : Vec::<Window>::new(),
            cnt : 0,
        };
        rt.draw();
        register_desktop(rt);
    }
    /// ### 处理输入、窗口销毁
    /// 此函数在 desktop.rs 的 run 函数中调用
    /// 检查每个窗口是否合法
    pub fn run(&mut self){
        self.get_input();
        // if !self.valid {
        //     // self.draw();
        //     // self.valid = true;
        // }
        self.refresh_window();
        self.refresh_dock()
    }
    /// ### 绘制所有桌面元素
    /// 没有进程同步（主要是 valid 变量），目前觉得影响不大
    pub fn draw(&mut self){
        if !self.valid{
            self.background.draw();
            for window in self.window.iter(){
                window.draw();
            }
            self.dock.draw();
            self.valid = true;
            invalid();
        }
    }
    pub fn setup_background(&mut self, path : &String){
        let file = open(path).unwrap();
        let image = generate_image(file).unwrap();
        self.background.set_texture(image);
    }
    /// ### 指定屏幕坐标创建命令行窗口
    pub fn create_terminal(&mut self, x : usize, y : usize){
        let window = Window::new(x, y, WINDOW_WIDTH, WINDOW_HEIGHT, WindowType::Text);
        self.window.push(window);
    }
    /// ### 指定屏幕坐标窗口文件窗口
    pub fn create_file_window(&mut self, x : usize, y : usize){
        let window = Window::new(x, y, WINDOW_WIDTH, WINDOW_HEIGHT, WindowType::File);
        self.window.push(window);
    }
    /// ### 选择这个点碰到的最表层的窗口
    pub fn select_window(&mut self, point : Point)->Option<usize>{
        let point = Position::from_point(point);
        if self.window.len() > 0 {
            for (idx, window) in self.window.iter_mut().rev().enumerate(){
                if window.hidden {
                    continue;
                }
                if window.detect(point) {
                    let len = self.window.len();
                    self.window.swap(len - 1, len - idx - 1);
                    return Some(len - 1);
                }
            }
            None
        }
        else {
            None
        }
    }
    pub fn refresh_window(&mut self){
        self.remove_window();
        for window in self.window.iter_mut() {
            window.refresh();
        }
    }
    pub fn refresh_dock(&mut self){
        loop {
            if let Some(ftype) = self.dock.pop_func() {
                match ftype {
                    FuncType::OpenTerminal => {
                        self.create_terminal((WIDTH - WINDOW_WIDTH) / 2, (HEIGHT - WINDOW_HEIGHT) / 2);
                        self.dock.add_window(self.window.last().unwrap().id, FuncType::TerminalWindow);
                        self.valid = false;
                    }
                    FuncType::OpenFolder => {
                        self.create_file_window((WIDTH - WINDOW_WIDTH) / 2, (HEIGHT - WINDOW_HEIGHT) / 2);
                        self.dock.add_window(self.window.last().unwrap().id, FuncType::TerminalWindow);
                        self.valid = false;
                    }
                    _ => {}
                }
            }
            else {
                break;
            }
        }
        loop {
            if let Some(id) = self.dock.pop_trigger() {
                self.trigger_window(id);
                self.valid = false;
            }
            else {
                break;
            }
        }
    }
    fn trigger_window(&mut self, id : usize) {
        for window in self.window.iter_mut() {
            if window.id == id {
                window.trigger_hidden();
                break;
            }
        }
    }
    /// ### 销毁已关闭窗口
    fn remove_window(&mut self){
        for (idx, t) in self.window.iter().enumerate() {
            if t.is_close() {
                self.dock.delete_window(t.id);
                self.window.remove(idx);
                self.valid = false;
                break;
            }
        }
    }
    /// ### 处理输入，转换成命令事件
    /// 按顺序获取按下、松开，鼠标位置信息，鼠标变量会在内部生成事件，接着处理鼠标事件
    fn get_input(&mut self){
        loop {
            let input = get_key_press();
            if input != 0{
                let key = Key::from(input);
                // println!("press {:?} {}", key, input);
                match key {
                    Key::MouseLeft | Key::MouseRight | Key::MouseMid => {
                        self.mouse.get_key_down(key);
                        self.valid = false;
                    }
                    _ => {
                        if let Some(term) = self.window.last_mut() {
                            term.input(key.to_char());
                            self.valid = false;
                        }
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
        loop {
            let input = get_scroll();
            if input != 0{
                self.mouse.scroll(input);
            }
            else{
                break;
            }
        }
        self.mouse.update_position();
        loop{
            let e = self.mouse.get_event();
            // println!("{:?}", e);
            if e == MouseEvent::None{
                break;
            }
            self.cmd(e);
        }
    }
    /// ### 处理鼠标事件
    fn cmd(&mut self, event : MouseEvent){
        match event {
            MouseEvent::RightClick => {
                let p = self.mouse.cur_pos.clone();
                self.create_terminal((p.x * WIDTH as f32) as usize, (p.y * HEIGHT as f32) as usize);
                self.valid = false;
            }
            MouseEvent::LeftClick => {
                let pos = self.mouse.pre_pos.clone();
                if let Some(_) = self.select_window(pos){
                    self.valid = false;
                }
                self.dock.detect(Position::from_point(pos));
            }
            MouseEvent::Drag => {
                let dir = self.mouse.get_move_dir();
                if let Some(idx) = self.select_window(self.mouse.pre_pos){
                    let term = self.window.get_mut(idx).unwrap();
                    term.translate(dir.0, dir.1);
                    self.valid = false;
                }
            }
            MouseEvent::ScrollDown => {
                if let Some(term) = self.window.last_mut(){
                    term.scroll_down();
                    self.valid = false;
                }
            }
            MouseEvent::ScrollUp => {
                if let Some(term) = self.window.last_mut(){
                    term.scroll_up();
                    self.valid = false;
                }
            }
            _ => {}
        }
    }
    
}

/// ## Dock

const DOCK_HEIGHT : usize = 60;
const DOCK_WIDTH : usize = 300;
const DOCK_ITEM_PADDING : usize = 5;

impl Dock {
    pub fn new()->Self{
        let x = (WIDTH - DOCK_WIDTH) / 2;
        let y = HEIGHT - DOCK_HEIGHT;
        let h = DOCK_HEIGHT / 2;
        let mut style = Style::new(ColorStyle::SolidColor, x, y + h, DOCK_WIDTH, h);
        style.set_color(Pixel{r:255, g:255, b:255, a:55});
        let mut rt = Self{
            button : Vec::<Button>::new(),
            function : Vec::<DockFunc>::new(),
            background : style,
            x : x,
            y : y,
            width : DOCK_WIDTH,
            height : DOCK_HEIGHT,
        };
        rt.add_func(FuncType::OpenFolder);
        rt.add_func(FuncType::OpenTerminal);
        rt
    }
    pub fn draw(&self){
        self.background.draw_blend();
        for btn in self.button.iter() {
            btn.draw_blend();
        }
    }
    pub fn pop_func(&mut self)->Option<FuncType>{
        for func in self.function.iter_mut() {
            if (func.ftype == FuncType::OpenFolder || func.ftype == FuncType::OpenTerminal)
                && func.trigger {
                func.trigger = false;
                return Some(func.ftype);
            }
        }
        None
    }
    pub fn pop_trigger(&mut self)->Option<usize>{
        for func in self.function.iter_mut() {
            if (func.ftype == FuncType::FolderWindow || func.ftype == FuncType::TerminalWindow)
                && func.trigger {
                func.trigger = false;
                return Some(func.id);
            }
        }
        None
    }
    pub fn add_window(&mut self, id : usize, wtype : FuncType){
        let len = self.function.len();
        let mut x = WIDTH / 2 - ((len + 1) * DOCK_HEIGHT + len * DOCK_ITEM_PADDING) / 2;
        let mut btn = Button::new(x, self.y - DOCK_ITEM_PADDING, DOCK_HEIGHT, DOCK_HEIGHT);
        let step = DOCK_HEIGHT + DOCK_ITEM_PADDING;
        if let Some(img) = self.get_image(wtype){
            btn.set_texture(img);
        }
        else {
            btn.set_color(Pixel::blue());
        }
        self.button.push(btn);
        self.function.push(DockFunc::new(wtype, id));
        for btn in self.button.iter_mut(){
            btn.set_position(x, self.y);
            x += step;
        }
    }
    pub fn delete_window(&mut self, id : usize) {
        for (i, func) in self.function.iter_mut().enumerate() {
            if func.id == id {
                self.function.remove(i);
                self.button.remove(i);
                self.refresh();
                break;
            }
        }
    }
    fn add_func(&mut self, ftype : FuncType){
        let len = self.function.len();
        let mut x = WIDTH / 2 - ((len + 1) * DOCK_HEIGHT + len * DOCK_ITEM_PADDING) / 2;
        let mut btn = Button::new(x, self.y - DOCK_ITEM_PADDING, DOCK_HEIGHT, DOCK_HEIGHT);
        let step = DOCK_HEIGHT + DOCK_ITEM_PADDING;
        if let Some(img) = self.get_image(ftype){
            btn.set_texture(img);
        }
        else {
            btn.set_color(Pixel::blue());
        }
        self.button.push(btn);
        self.function.push(DockFunc::new(ftype, 0));
        for btn in self.button.iter_mut(){
            btn.set_position(x, self.y);
            x += step;
        }
    }
    fn get_image(&self, ftype : FuncType)->Option<Image>{
        let path;
        if ftype == FuncType::OpenTerminal || ftype == FuncType::TerminalWindow {
            path = "0/img/terminal_color.bmp".to_string();
        }
        else if ftype == FuncType::OpenFolder || ftype == FuncType::FolderWindow {
            path = "0/img/folder.bmp".to_string();
        }
        else {
            return None;
        }
        if let Some(mut file) = open(&path) {
            if file.open(OpenFlag::Read.val()).is_ok(){
                return generate_image(file);
            }
        }
        None
    }
}

impl Transform for Dock {
    fn set_position(&mut self, x : usize, y : usize) {
        self.x = x;
        self.y = y;
        self.background.set_position(x, y);
        for btn in self.button.iter_mut() {
            btn.set_position(x, y);
        }
    }

    fn translate(&mut self, x : isize, y : isize) {
        let mut xx = self.x as isize + x;
        let mut yy = self.y as isize + y;
        if xx < 0{
            xx = 0;
        }
        if yy < 0{
            yy = 0;
        }
        self.x = xx as usize;
        self.y = yy as usize;
        for btn in self.button.iter_mut(){
            btn.translate(x, y);
        }
        self.background.translate(x, y);
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
            for (idx, btn) in self.button.iter_mut().enumerate() {
                if btn.detect(point) {
                    btn.click = false;
                    let func = self.function.get_mut(idx).unwrap();
                    func.trigger = true;
                }
            }
        }
        rt
    }

    fn refresh(&mut self){
        self.button.clear();
        let funcs = self.function.clone();
        self.function.clear();
        for func in funcs.iter() {
            self.add_func(func.ftype);
        }
    }
}

/// ## 窗口
/// 窗口同时承载文件显示和命令行的功能

static mut WINDOW_ID : usize = 1;

impl Window {
    pub fn new(x : usize, y : usize, width : usize, height : usize, ctype : WindowType)->Self{
        let id;
        unsafe {
            id = WINDOW_ID;
            WINDOW_ID += 1;
        }
        let mut shell = InterShell::new();
        shell.filetree = get_root(0);
        let mut content = WindowContent::new(x, y + HEADBAR_HEIGHT, width, height - HEADBAR_HEIGHT, ctype);
        if ctype == WindowType::File {
            if let Some(s) = &shell.filetree{
                content.refresh_content(s);
            }
        }
        let rt = Self{
            x : x,
            y : y,
            width : width,
            height : height,
            hidden : false,
            head_bar : HeadBar::new(x, y, width, HEADBAR_HEIGHT),
            content : content,
            shell : shell,
            id : id,
        };
        rt
    }
    pub fn draw(&self){
        if self.hidden {
            return;
        }
        self.head_bar.draw();
        self.content.draw();
    }
    /// ### 判断是否已经关闭
    /// 窗口的关闭依赖于关闭按钮的状态
    pub fn is_close(&self)->bool{
        self.head_bar.close_button.click
    }
    /// ### 传递输入给窗口的 Shell
    /// 非命令行不接受输入，边接受边显示
    /// 同时获取 Shell 处理得到的结果输出
    pub fn input(&mut self, c : char){
        if self.content.ctype != WindowType::Text{
            return;
        }
        self.shell.input(c);
        self.content.putchar(c);
        for c in self.shell.pop_output(){
            self.content.putchar_color(c.c, c.color);
        }
    }
    /// ### 处理滚轮事件
    pub fn scroll_down(&mut self){
        self.content.canvas.scroll(-16);
    }
    pub fn scroll_up(&mut self){
        self.content.canvas.scroll(16);
    }
    pub fn is_hidden(&self)->bool {
        self.head_bar.is_hidden()
    }
    pub fn trigger_hidden(&mut self) {
        self.head_bar.trigger_hidden();
        self.hidden = self.is_hidden();
    }

}

impl Transform for Window {
    fn set_position(&mut self, x : usize, y : usize) {
        self.head_bar.set_position(x, y);
        self.content.set_position(x, y);
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

    fn translate(&mut self, x : isize, y : isize) {
        let mut x = x;
        let mut y = y;
        let mut xx = self.x as isize + x;
        let mut yy = self.y as isize + y;
        if xx < 0{
            xx = 0;
            x = - (self.x as isize);
        }
        if yy < 0{
            yy = 0;
            y = - (self.y as isize);
        }
        self.x = xx as usize;
        self.y = yy as usize;
        self.head_bar.translate(x, y);
        self.content.translate(x, y);
    }

    fn refresh(&mut self) {
        self.hidden = self.is_hidden();
    }
}

/// ## 窗口内容区域
/// 命令行与文件窗口的区别体现在此

const FILE_RECT_SIZE : usize = 80;
pub const FONT_HEIGHT : usize = 16;
pub const FONT_WIDTH : usize = 8;
const FILE_PADDING : usize = 20;
const FILE_RECT_COLOR : Pixel = Pixel::grey();
const FILE_BACKGROUND : Pixel = Pixel::white();

impl WindowContent {
    pub fn new(x : usize, y : usize, width : usize, height : usize, ctype : WindowType)->Self{
        let mut canvas = Canvas::new(x, y, width, height);
        if ctype == WindowType::Text{
            canvas.fill(Pixel::black());
        }
        else {
            println!("window set color");
            canvas.fill(Pixel::white());
        }
        let mut rt = Self{
            ctype : ctype,
            width : width,
            height : height,
            canvas : canvas,
            write_x : 0,
            write_y : 0,
        };
        if ctype == WindowType::Text{
            rt.putchar_color('=', Pixel::green());
            rt.putchar_color(':', Pixel::green());
        }
        rt
    }
    pub fn draw(&self){
        self.canvas.draw();
    }
    /// ## 命令行功能
    /// ### 默认颜色输出 ASCII 字符
    pub fn putchar(&mut self, c : char){
        self.putchar_color(c, Pixel::white());
    }
    /// ### 指定颜色输出 ASCII 字符
    pub fn putchar_color(&mut self, c : char, color : Pixel){
        if c == '\n' || c == '\r' {
            self.write_x = 0;
            self.write_y += FONT_HEIGHT;
            self.putchar_color('=', Pixel::green());
            self.putchar_color(':', Pixel::green());
            return;
        }
        self.canvas.fill_font(c as usize, self.write_x, self.write_y,
            color, Pixel::black());
        self.write_x += 8;
        self.write_y += ((self.write_x + 8) / self.width as usize) * FONT_HEIGHT;
        if self.write_x + 8 > self.width as usize{
            self.write_x = 0;
        }
    }
    /// ## 文件窗口功能
    /// ### 刷新文件显示
    pub fn refresh_content(&mut self, tree : &FileTree){
        assert!(self.ctype == WindowType::File);
        self.write_x = 0;
        self.write_y = 0;
        self.canvas.fill(FILE_BACKGROUND);
        for item in tree.items.iter(){
            self.add(item);
        }
    }
    /// ### 增加一个文件显示单元
    fn add(&mut self, item : &TreeItem){
        if self.write_x + FILE_RECT_SIZE >= self.width as usize {
            self.write_y += FILE_RECT_SIZE;
            self.write_x = 0;
        }
        let rect = Rect {
            x1 : (self.write_x + FILE_PADDING) as u32,
            y1 : (self.write_y + FILE_PADDING) as u32,
            x2 : (self.write_x + FILE_RECT_SIZE - FILE_PADDING) as u32,
            y2 : (self.write_y + FILE_RECT_SIZE - FILE_PADDING) as u32,
        };
        let s;
        if item.is_file() {
            s = &"0/img/file_black.bmp";
        }
        else{
            s = &"0/img/folder_black.bmp";
        }
        if let Some(mut file) = open(&s.to_string()){
            file.open(OpenFlag::Read.val()).ok();
            if let Some(img) = generate_image(file){
                self.canvas.fill_image(rect, img);
            }
            else {
                self.canvas.fill_rect(rect, FILE_RECT_COLOR);
            }
        }
        else {
            self.canvas.fill_rect(rect, FILE_RECT_COLOR);
        }
        let mut x;
        if FILE_RECT_SIZE > FONT_WIDTH * item.filename.len() {
            x = self.write_x + (FILE_RECT_SIZE - FONT_WIDTH * item.filename.len()) / 2;
        }
        else {
            x = self.write_x;
        }
        let y = self.write_y + FILE_RECT_SIZE - FILE_PADDING;
        for c in item.filename.bytes() {
            self.canvas.fill_font(c as usize, x, y, Pixel::black(), Pixel::white());
            if x >= self.write_x + FILE_RECT_SIZE as usize {
                break;
            }
            x += FONT_WIDTH;
        }
        self.write_x += FILE_RECT_SIZE;
    }
}

impl Transform for WindowContent {
    fn set_position(&mut self, x : usize, y : usize) {
        self.canvas.element.set_position(x, y);
    }

    fn maximum(&mut self) {
    }

    fn minimum(&mut self) {
    }

    fn detect(&mut self, point : Position)->bool {
        self.canvas.detect(point)
    }

    fn translate(&mut self, x : isize, y : isize) {
        self.canvas.translate(x, y);
    }

    fn refresh(&mut self) {
    }
}

/// ## 标题栏

const HEADBAR_HEIGHT : usize = 20;

impl HeadBar {
    pub fn new(x : usize, y : usize, width : usize, height : usize)->Self{
        let close_btn = Button::new(x + width - BUTTON_WIDTH, y, BUTTON_WIDTH, height);
        let mut small_btn = Button::new(x + width - BUTTON_WIDTH * 2, y, BUTTON_WIDTH, height);
        small_btn.set_color(Pixel::shallow_grey());
        let mut back = Style::new(ColorStyle::SolidColor, x, y, width, height);
        back.set_color(Pixel::grey());
        
        Self{
            background : back,
            close_button : close_btn,
            small_button : small_btn,
            x : x,
            y : y,
            width : width,
            height : height,
            button_width : 30,
            close_button_offset : 470,
        }
    }
    pub fn draw(&self){
        self.background.draw();
        self.close_button.draw();
        self.small_button.draw();
    }
    pub fn is_hidden(&self)->bool {
        self.small_button.click
    }
    pub fn trigger_hidden(&mut self){
        self.small_button.click = !self.small_button.click;
    }
}

impl Transform for HeadBar {
    fn set_position(&mut self, x : usize, y : usize) {
        self.x = x;
        self.y = y;
        self.background.set_position(x, y);
        self.close_button.set_position(x + self.width - BUTTON_WIDTH, y);
        self.small_button.set_position(x + self.width - BUTTON_WIDTH * 2, y);
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
            self.small_button.detect(point);
        }
        rt
    }

    fn translate(&mut self, x : isize, y : isize) {
        let mut xx = self.x as isize + x;
        let mut yy = self.y as isize + y;
        if xx < 0{
            xx = 0;
        }
        if yy < 0{
            yy = 0;
        }
        self.x = xx as usize;
        self.y = yy as usize;
        self.background.translate(x, y);
        self.close_button.translate(x, y);
        self.small_button.translate(x, y);
    }

    fn refresh(&mut self) {
    }
}

use crate::{filesystem::{file::OpenFlag, file_tree::{FileTree, TreeItem}, image::{bmp::generate_image, image::Image}, interface::open, operation::get_root}, graphic::{canvas::Canvas, element::Draw, transform::ElemTranform}, interact::shell::{InterShell}, virtio::{gpu_device::{HEIGHT, Pixel, Rect, WIDTH, invalid}, input::{input_buffer::{Point, get_key_press, get_key_release, get_scroll}, keyboard::Key}}};
use crate::uart;
use super::{controll::{button::{BUTTON_WIDTH, Button}, style::style::{ColorStyle, Style}}, desktop::{Desktop, Dock, DockFunc, FuncType, HeadBar, Position, Window, WindowContent, WindowType, register_desktop}, desktop_trait::{Transform}, mouse::{Mouse, MouseEvent}};
use alloc::{prelude::v1::*};
