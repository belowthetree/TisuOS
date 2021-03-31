//! # 显示驱动
//! 
//! 2021年3月30日

#![allow(dead_code)]
use core::{cmp::min, mem::size_of};

use crate::{libs::{graphic::Pixel, shape::Rect}, memory::{alloc, alloc_kernel_page, config::PAGE_SIZE, free}, sync::{Bool, Mutex}};

use super::{header::VirtHeader, queue::{DescFlag, VIRTIO_DESC_F_NEXT, VIRTIO_RING_SIZE, VirtQueue}};

pub struct GPU{
	header : &'static mut VirtHeader,
	queue : &'static mut VirtQueue,
    pub pin_idx : usize,
    frame_buffer : *mut Pixel,
    width : usize,
    height : usize,
    mutex : Mutex,
    int : Bool,
}

impl GPU {
    pub fn new(idx : usize, header : *mut VirtHeader, width : usize, height : usize)->Self{
		let num = (size_of::<VirtQueue>() + PAGE_SIZE - 1) / PAGE_SIZE;
		let queue = alloc_kernel_page(num).unwrap() as *mut VirtQueue;
		let header = unsafe {&mut *header};
        let num = (width * height * size_of::<Pixel>() + PAGE_SIZE - 1) / PAGE_SIZE;
		header.set_feature(!0).unwrap();
		header.set_ring_size(VIRTIO_RING_SIZE as u32).unwrap();
		header.set_page_size(PAGE_SIZE as u32);
		header.set_pfn(0, queue);
		header.driver_ok();

        let mut rt = Self{
			header : header,
			queue : unsafe {&mut *queue},
            pin_idx: idx,
            frame_buffer: alloc_kernel_page(num).unwrap() as *mut Pixel,
            width: width,
            height: height,
            mutex: Mutex::new(),
            int: Bool::new(),
        };
        rt.reset();
        rt
    }
    /// 清空屏幕 rgba（10，10，10，255）
    fn reset(&mut self){
        let rect = Rect{x1:0,y1:0,x2:self.width as u32,y2:self.height as u32};
        self.fill_rect(0, 0, self.width, self.height, Pixel{r:10,g:10,b:10,a:255});
        self.create_resouce_id(self.width, self.height, 1);
        self.attach(1);
        self.set_scanout(rect.clone(), 1, 0);
        self.transfer(rect.clone(), 1);
        self.flush(rect.clone(), 1);
        self.run();
    }
    /// 刷新 resouce 里的区域
    pub fn flush(&mut self, rect : Rect, resource_idx : usize){
        let flush = ResourceFlush::new(rect, resource_idx);
        self.add_desc::<ResourceFlush>(flush as u64, ControllType::ResourceFlush);
    }
    /// 将 source 对应区域传输给 GPU
    pub fn transfer(&mut self, rect : Rect, resource_idx : usize){
        let trans = TransferToHost2d::new(rect, resource_idx);
        self.add_desc::<TransferToHost2d>(trans as u64, ControllType::TransferToHost2d);
    }
    /// 将 source 和 scanout 中的某个区域绑定
    pub fn set_scanout(&mut self, rect : Rect, resource_idx : usize, scanout_idx : usize){
        let scan = Scanout::new(rect, resource_idx, scanout_idx);
        self.add_desc::<Scanout>(scan as u64, ControllType::SetScanout);
    }
    /// 创建一个 source，设定好宽、高
    pub fn create_resouce_id(&mut self, width : usize, height : usize, resource_idx : usize){
        let create = Create2D::new(width, height, resource_idx);
        self.add_desc::<Create2D>(create as u64, ControllType::ResourceCreate2d);
    }

    fn add_desc<T>(&mut self, addr1 : u64, ctype : ControllType) {
        let header = ControllHeader::new();
        unsafe {(*header).ctype = ctype;}

        let ref mut q = self.queue;
        q.add_avail();
        q.add_desc(addr1, size_of::<T>() as u32, DescFlag::Next as u16);
        q.add_desc(header as u64, size_of::<ControllHeader>() as u32,
        DescFlag::Write as u16);

    }

    /// 将 source 与某块内存绑定
    pub fn attach(&mut self, resource_idx : u32){
        let at = AttachBacking::new(resource_idx, resource_idx);
        let entry = self.entry();
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::ResourceAttachBacking;
        }
        let ref mut q = self.queue;
        q.add_avail();
        q.add_desc(at as u64, size_of::<AttachBacking>() as u32,
        DescFlag::Next as u16);
        q.add_desc(entry as u64, size_of::<MemEntry>() as u32,
        DescFlag::Next as u16);
        q.add_desc(header as u64, size_of::<ControllHeader>() as u32,
        DescFlag::Write as u16);
    }
    /// 发送 QueueNotify
    fn run(&mut self){
        self.header.notify();
    }

    fn entry(&self)->*const MemEntry{
        let entry = alloc(size_of::<MemEntry>(), true).unwrap() as *mut MemEntry;
        unsafe {
            (*entry).addr = self.frame_buffer as u64;
            (*entry).length = (self.width * self.height * size_of::<Pixel>()) as u32;
            (*entry).padding = 0;
        }
        entry
    }

    fn fill_rect(&mut self, x : usize, y : usize, width : usize, height : usize, color : Pixel){
        for i in x..(x + width){
            for j in y..(y + height){
                unsafe {
                    let idx = j * self.width + i;
                    self.frame_buffer.add(idx).write_volatile(color);
                }
            }
        }
    }

    pub fn pending(&mut self) {
        self.int.set_true();
    }

    pub fn interrupt_handler(&mut self) {
        if !self.int.pop() {
            return;
        }
        unsafe {
            while self.queue.is_pending() {
                let elem = self.queue.next_elem();
                let mut idx = elem.id as usize;
                while self.queue.desc[idx].flags & VIRTIO_DESC_F_NEXT != 0 {
                    free(self.queue.desc[idx].addr as *mut u8);
                    idx = self.queue.desc[idx].next as usize;
                }
                use crate::uart;
                if (*(self.queue.desc[idx].addr as *const ControllHeader)).ctype != ControllType::RespOkNoData{
                    println!("GPU Err {:?}", (*(self.queue.desc[idx].addr as *const ControllHeader)).ctype);
                }
                free(self.queue.desc[idx].addr as *mut u8);
            }
            self.mutex.unlock();
        }
    }

    pub fn draw_rect_override(&mut self, rect : Rect, color_buffer : *mut Pixel){
        if rect.x1 as usize >= self.width || rect.y1 as usize > self.height{
            return;
        }
        let st = rect.y1 as usize * self.width;
        let ed = min(rect.y2 as usize, self.height) * self.width;
        let ptr = color_buffer;
        let mut idx;
        let mut row = 0;
        let width = (rect.x2 - rect.x1) as usize;
        let line = (min(rect.x2, self.width as u32) - rect.x1) as usize;
        self.mutex.lock();
        for y in (st..ed).step_by(self.width){
            idx = row * width;
            unsafe {
                self.frame_buffer.add(y + rect.x1 as usize).copy_from(ptr.add(idx), line);
            }
            row += 1;
        }
        self.mutex.unlock();
    }
    /// ### 透明度着色
    /// 包含大量浮点数运算，速度较慢
    pub fn draw_rect_blend(&mut self, rect : Rect, color_buffer : *mut Pixel){
        if rect.x1 as usize >= self.width || rect.y1 as usize > self.height{
            return;
        }
        let st = rect.y1 as usize * self.width;
        let ed = min(rect.y2 as usize, self.height) * self.width;
        let ptr = color_buffer as *const Pixel;
        let mut idx = 0;
        self.mutex.lock();
        let t = 1.0 / 255.0;
        for y in (st..ed).step_by(self.width){
            for x in rect.x1..min(rect.x2, self.width as u32){
                unsafe {
                    let id = x as usize + y;
                    let color1 = *ptr.add(idx);
                    let color2 = *self.frame_buffer.add(id);
                    let rate =  color1.a as f32 * t;
                    let rate2 = 1.0 - rate;
                    let color = Pixel{
                        r : (color1.r as f32 * rate) as u8 + (color2.r as f32 * rate2) as u8,
                        g : (color1.g as f32 * rate) as u8 + (color2.g as f32 * rate2) as u8,
                        b : (color1.b as f32 * rate) as u8 + (color2.b as f32 * rate2) as u8,
                        a : (color1.a as f32 * rate) as u8 + (color2.a as f32 * rate2) as u8,
                    };
                    self.frame_buffer.add(id).write_volatile(color);
                    idx += 1;
                }
            }
        }
        self.mutex.unlock();
    }    

    pub fn refresh(&mut self) {
        let rect = Rect{x1:0, y1:0, x2:self.width as u32, y2:self.height as u32};
        self.transfer(rect, 1);
        self.flush(rect, 1);
        self.run();
    }
}



#[repr(C)]
struct ResourceFlush {
	header : ControllHeader,
	rect : Rect,
	resource_id : u32,
	padding : u32,
}
#[repr(C)]
#[derive(Debug)]
pub struct ControllHeader{
    ctype : ControllType,
    flag : u32,
    fence_idx : u64,
    ctx_id : u32,
    padding : u32,
}
#[repr(C)]
struct Scanout {
	header: ControllHeader,
	rect: Rect,
	scanout_id: u32,
	resource_id: u32,
}
#[repr(C)]
pub struct Create2D{
    header : ControllHeader,
    resource_idx : u32,
    format : PixelFormat,
    width : u32,
    height : u32,
}
#[repr(C)]
struct TransferToHost2d {
	header: ControllHeader,
	rect: Rect,
	offset: u64,
	resource_id: u32,
	padding: u32,
}
#[repr(C)]
struct AttachBacking {
	header: ControllHeader,
	resource_id: u32,
	entries: u32,
}
#[repr(C)]
struct MemEntry {
	addr: u64,
	length: u32,
	padding: u32,
}

#[repr(u32)]
#[derive(Debug, PartialEq)]
pub enum ControllType {
    GetDisplayInfo = 0x0100,
	ResourceCreate2d,
	ResourceUref,
	SetScanout,
	ResourceFlush,
	TransferToHost2d,
	ResourceAttachBacking,
	ResourceDetachBacking,
	GetCapsetInfo,
	GetCapset,
	GetEdid,
	// cursor commands
	UpdateCursor = 0x0300,
	MoveCursor,
	// success responses
	RespOkNoData = 0x1100,
	RespOkDisplayInfo,
	RespOkCapsetInfo,
	RespOkCapset,
	RespOkEdid,
	// error responses
	RespErrUnspec = 0x1200,
	RespErrOutOfMemory,
	RespErrInvalidScanoutId,
	RespErrInvalidResourceId,
	RespErrInvalidContextId,
	RespErrInvalidParameter,
}

#[allow(dead_code)]
#[repr(C)]
enum PixelFormat {
    B8G8R8A8 = 1,
    B8G8R8X8 = 2,
    A8R8G8B8 = 3,
    X8R8G8B8 = 4,
    R8G8B8A8 = 67,
    X8B8G8R8 = 68,
    A8B8G8R8 = 121,
    R8G8B8X8 = 134,
}

impl Create2D {
    pub fn new(width : usize, height : usize, resouce_idx : usize)->*const Create2D{
        let rect = alloc(size_of::<Create2D>(), true).unwrap() as *mut Create2D;
        unsafe {
            (*rect).header = ControllHeader{
                ctype : ControllType::ResourceCreate2d,
                ctx_id : 0,
                fence_idx : 0,
                flag : 0,
                padding : 0,
            };
            (*rect).resource_idx = resouce_idx as u32;
            (*rect).format = PixelFormat::R8G8B8A8;
            (*rect).width = width as u32;
            (*rect).height = height as u32;
        }
        rect
    }
}

impl ControllHeader {
    pub fn new()->*mut ControllHeader{
        alloc(size_of::<ControllHeader>(), true).unwrap() as *mut ControllHeader
    }
}

impl TransferToHost2d {
    pub fn new(rect : Rect, resouce_idx : usize)->*const TransferToHost2d{
        let rt = alloc(size_of::<TransferToHost2d>(), true).unwrap() as *mut TransferToHost2d;
        unsafe {
            (*rt).header = ControllHeader{
                ctype : ControllType::TransferToHost2d,
                flag : 0,
                fence_idx : 0,
                ctx_id : 0,
                padding : 0,
            };
            (*rt).rect = rect;
            (*rt).offset = 0;
            (*rt).resource_id = resouce_idx as u32;
            (*rt).padding = 0;
        }
        rt
    }
}

impl AttachBacking {
    pub fn new(resource_idx : u32, entries : u32) ->*const AttachBacking{
        let attach = alloc(size_of::<AttachBacking>(), true).unwrap() as *mut AttachBacking;
        unsafe {
            (*attach).header = ControllHeader{
                ctype : ControllType::ResourceAttachBacking,
                flag : 0,
                fence_idx : 0,
                ctx_id : 0,
                padding : 0,
            };
            (*attach).resource_id = resource_idx;
            (*attach).entries = entries;
        }
        attach
    }
}

impl Scanout {
    pub fn new(rect : Rect, resource_idx : usize, scanout_idx : usize)->*const Scanout{
        let sc = alloc(size_of::<Scanout>(), true).unwrap() as *mut Scanout;
        unsafe {
            (*sc).header = ControllHeader{
                ctype : ControllType::SetScanout,
                flag : 0,
                fence_idx : 0,
                ctx_id : 0,
                padding : 0,
            };
            (*sc).rect = rect;
            (*sc).resource_id = resource_idx as u32;
            (*sc).scanout_id = scanout_idx as u32;
        }
        sc
    }
}

impl ResourceFlush {
    pub fn new(rect : Rect, resouce_idx : usize)->*mut ResourceFlush{
        let rt = alloc(size_of::<ResourceFlush>(), true).unwrap() as *mut ResourceFlush;
        unsafe {
            (*rt).header = ControllHeader{
                ctype : ControllType::ResourceFlush,
                ctx_id : 0,
                flag : 0,
                fence_idx : 0,
                padding : 0,
            };
            (*rt).rect = rect;
            (*rt).resource_id = resouce_idx as u32;
            (*rt).padding = 0;
        }
        rt
    }
}


