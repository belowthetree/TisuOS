//! # 桌面
//! 维护一个桌面实例，底层依赖于 Graphic
//! 2020年12月30日 zg

#![allow(dead_code)]

static mut DESKTOP_LOCK : Mutex = Mutex::new();
static mut DESKTOP : Option<Vec<Desktop>> = None;

/// ## 桌面管理者
pub struct Desktop{
    pub valid : bool,
    pub mouse : Mouse,
    pub background : Style,
    pub dock : Dock,
    pub window : Vec<Window>,
    pub cnt : usize,
}

/// ## 颜色字符
#[derive(Clone, Copy)]
pub struct ColorChar{
    pub color : Pixel,
    pub c : char,
}
/// ## 窗口
/// 命令行或文件窗口，由 `ctype` 域指出
pub struct Window{
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
    pub hidden : bool,
    pub head_bar : HeadBar,
    pub content : WindowContent,
    pub shell : InterShell,
    pub id : usize,
}
/// ## 窗口类型
#[derive(PartialEq, Clone, Copy)]
pub enum WindowType {
    Text,
    File,
}
/// ## 窗口内容
/// 命令行显示文字
/// 文件窗口图形化显示目录信息
pub struct WindowContent{
    pub ctype : WindowType,
    pub width : usize,
    pub height : usize,
    pub canvas : Canvas,
    pub write_x : usize,
    pub write_y : usize,
}
/// ## 标题栏
/// 包含按钮，管理窗口按钮
pub struct HeadBar{
    pub background : Style,
    pub close_button : Button,
    pub small_button : Button,
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
    pub button_width : usize,
    pub close_button_offset : usize,
}
/// ## 程序坞
pub struct Dock{
    pub button : Vec<Button>,
    pub function : Vec<DockFunc>,
    pub background : Style,
    pub x : usize,
    pub y : usize,
    pub width : usize,
    pub height : usize,
}
#[derive(Clone, Copy)]
pub struct DockFunc {
    pub ftype : FuncType,
    pub id : usize, // 如果是窗口，则保存 id
    pub trigger : bool,
}
impl DockFunc {
    pub fn new(ftype : FuncType, id : usize)->Self {
        Self {
            ftype : ftype,
            id : id,
            trigger : false,
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
pub enum FuncType{
    OpenFolder,
    OpenTerminal,
    FolderWindow,
    TerminalWindow,
}
/// ## 屏幕坐标
/// 桌面元素统一使用，支持 Point 的转换
#[derive(Clone, Copy)]
pub struct Position {
    pub x : usize,
    pub y : usize,
}

impl Position {
    pub fn from_point(point : Point)->Self{
        let x = (point.x * WIDTH as f32) as usize;
        let y = (point.y * HEIGHT as f32) as usize;
        Self{
            x : x,
            y : y,
        }
    }
}

pub fn init(){
    unsafe {
        DESKTOP = Some(Vec::<Desktop>::new());
        // register(draw);
    }
}
/// ## 每个时间片驱动桌面逻辑
pub fn run(){
    unsafe {
        loop{
            asm!("wfi"::::"volatile");
            DESKTOP_LOCK.lock();
            if let Some(desk) = &mut DESKTOP{
                for desktop in desk.iter_mut(){
                    desktop.run();
                    if !desktop.valid{
                        desktop.draw();
                    }
                }
            }
            DESKTOP_LOCK.unlock();
        }
    }
}
/// ## 提供给输入中断的绘制函数
pub fn draw(){
    unsafe {
        if let Some(desk) = &mut DESKTOP{
            for desktop in desk.iter_mut(){
                desktop.draw();
            }
        }
    }
}
/// ## 注册桌面
pub fn register_desktop(desktop : Desktop){
    unsafe {
        DESKTOP_LOCK.lock();
        if let Some(desk) = &mut DESKTOP{
            desk.push(desktop);
        }
        DESKTOP_LOCK.unlock();
    }
}

use alloc::{prelude::v1::*};
// use crate::uart;
use crate::{graphic::canvas::Canvas, interact::shell::InterShell,
    sync::Mutex, virtio::{gpu_device::{HEIGHT, Pixel, WIDTH}, input::input_buffer::{Point}}};

use super::{controll::{button::Button, style::style::{Style}}, mouse::Mouse};

