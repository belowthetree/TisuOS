//! # GPU
//! GPU 控制接口
//! 
//! 2020年12月28日 zg

#![allow(dead_code)]
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
#[derive(Debug)]
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
pub struct GPU{
    queue : *mut Queue,
    device_idx : usize,
    ptr : *mut u32,
    queue_idx : usize,
    used_idx : usize,
    frame_buffer : *mut Pixel,
    rect : Rect,
    width : usize,
    height : usize,
    display_idx : usize,
    write_idx : usize,
    mutex : Mutex,
    valid : bool,
}

impl GPU {
    pub fn new(queue : *mut Queue, ptr : *mut u32, idx : usize)->Self{
        let n = (WIDTH * HEIGHT * size_of::<Pixel>() + PAGE_SIZE - 1) / PAGE_SIZE + 2;
        let addr = alloc_kernel_page(n);
        println!("gpu at {:x}", addr as usize);
        Self{
            queue : queue,
            device_idx : idx,
            ptr : ptr,
            queue_idx : 0,
            used_idx : 0,
            frame_buffer : addr as *mut Pixel,
            rect : Rect{x1:0,x2:0,y1:0,y2:0},
            width : WIDTH,
            height : HEIGHT,
            display_idx : 1,
            write_idx : 2,
            mutex : Mutex::new(),
            valid : true,
        }
    }
    /// 清空屏幕 rgba（10，10，10，255）
    pub fn reset(&mut self){
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
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::ResourceFlush;
        }
        let head_idx = self.queue_idx;
        self.add_desc(flush as u64, size_of::<ResourceFlush>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(header as u64, size_of::<ControllHeader>() as u32, VIRTIO_DESC_F_WRITE);
        self.add_ring(head_idx);
    }
    /// 将 source 对应区域传输给 GPU
    pub fn transfer(&mut self, rect : Rect, resource_idx : usize){
        let trans = TransferToHost2d::new(rect, resource_idx);
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::TransferToHost2d;
        }
        let head_idx = self.queue_idx;
        self.add_desc(trans as u64, size_of::<TransferToHost2d>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(header as u64, size_of::<ControllHeader>() as u32, VIRTIO_DESC_F_WRITE);
        self.add_ring(head_idx);
    }
    /// 将 source 和 scanout 中的某个区域绑定
    pub fn set_scanout(&mut self, rect : Rect, resource_idx : usize, scanout_idx : usize){
        let scan = Scanout::new(rect, resource_idx, scanout_idx);
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::SetScanout;
        }
        let head_idx = self.queue_idx;
        self.add_desc(scan as u64, size_of::<Scanout>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(header as u64, size_of::<ControllHeader>() as u32, VIRTIO_DESC_F_WRITE);
        self.add_ring(head_idx);
    }
    /// 创建一个 source，设定好宽、高
    pub fn create_resouce_id(&mut self, width : usize, height : usize, resource_idx : usize){
        let rect = Create2D::new(width, height, resource_idx);
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::ResourceCreate2d;
        }
        let head_idx = self.queue_idx;
        self.add_desc(rect as u64, size_of::<Create2D>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(header as u64, size_of::<ControllHeader>() as u32, VIRTIO_DESC_F_WRITE);
        self.add_ring(head_idx);
    }
    pub fn invalid(&mut self){
        self.mutex.lock();
        self.valid = false;
        self.mutex.unlock();
    }
    /// 将 source 与某块内存绑定
    pub fn attach(&mut self, resource_idx : u32){
        let at = AttachBacking::new(resource_idx, resource_idx);
        let entry = self.entry();
        let header = ControllHeader::new();
        unsafe {
            (*header).ctype = ControllType::ResourceAttachBacking;
        }
        let head_idx = self.queue_idx;
        self.add_desc(at as u64, size_of::<AttachBacking>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(entry as u64, size_of::<MemEntry>() as u32, VIRTIO_DESC_F_NEXT);
        self.add_desc(header as u64, size_of::<ControllHeader>() as u32, VIRTIO_DESC_F_WRITE);
        self.add_ring(head_idx);
    }
    /// 覆盖着色
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
    /// 透明度着色
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
    /// 发送 QueueNotify
    fn run(&mut self){
        unsafe {
            // println!("{:x} {}", self.ptr as usize, Offset::QueueNotify.scale32());
            *(self.ptr.add(Offset::QueueNotify.scale32())) = 0;
        }
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
    fn add_ring(&mut self, head_idx : usize){
        unsafe {
            //println!("head {}, avail {}", head_idx, (*self.queue).avail.idx);
            (*self.queue).avail.ring[(*self.queue).avail.idx as usize % VIRTIO_RING_SIZE] = head_idx as u16;
            (*self.queue).avail.idx = (*self.queue).avail.idx.wrapping_add(1);
        }
    }
    fn add_desc(&mut self, addr : u64, len : u32, flag : u16){
        let idx = self.queue_idx;
        self.queue_idx = (self.queue_idx + 1) % VIRTIO_RING_SIZE;
        let next = if flag & VIRTIO_DESC_F_NEXT != 0 {self.queue_idx as u16} else { 0 };
        unsafe {
            (*self.queue).desc[idx] = Descriptor{
                addr : addr,
                len : len,
                flags : flag,
                next : next,
            };
        }
    }
    pub fn fill_rect(&mut self, x : usize, y : usize, width : usize, height : usize, color : Pixel){
        for i in x..(x + width){
            for j in y..(y + height){
                unsafe {
                    let idx = j * self.width + i;
                    self.frame_buffer.add(idx).write_volatile(color);
                }
            }
        }
    }

}

pub const WIDTH : usize = 640;
pub const HEIGHT : usize = 480;
pub static mut DEVICE : Option<Vec<GPU>> = None;

pub fn init(){
    unsafe {
        DEVICE = Some(Vec::<GPU>::new());
    }
}

pub fn init_gpu(ptr : *mut u32, idx : usize) ->bool {
    unsafe {
        ptr.add(Offset::Status.scale32()).write_volatile(0);
        let mut status = StatusField::Acknowledge.val32();
        ptr.add(Offset::Status.scale32()).write_volatile(status);
        status |= StatusField::Driver.val32();
        ptr.add(Offset::Status.scale32()).write_volatile(status);
        let feat = ptr.add(Offset::HostFeatures.scale32()).read_volatile();
        ptr.add(Offset::GuestFeatures.scale32()).write_volatile(feat);
        status |= StatusField::FeaturesOk.val32();
        ptr.add(Offset::Status.scale32()).write_volatile(status);
        let res = ptr.add(Offset::Status.scale32()).read_volatile();
        if res & StatusField::FeaturesOk.val32() == 0{
            println!("feature fail");
            ptr.add(Offset::Status.scale32()).write_volatile(StatusField::Failed.val32());
            return false;
        }
        let num_max = ptr.add(Offset::QueueNumMax.scale32()).read_volatile();
        if num_max < VIRTIO_RING_SIZE as u32 {
            println!("Queue num too larget {} < {}", num_max, VIRTIO_RING_SIZE);
            return false;
        }
        ptr.add(Offset::GuestPageSize.scale32()).write_volatile(PAGE_SIZE as u32);
        // 设置队列
        let num_page = (size_of::<Queue>() + PAGE_SIZE - 1) / PAGE_SIZE;
        let queue = alloc_kernel_page(num_page) as usize;
        let pfn = (queue / PAGE_SIZE) as u32;
        ptr.add(Offset::QueueSel.scale32()).write_volatile(0);
        ptr.add(Offset::QueueNum.scale32()).write_volatile(VIRTIO_RING_SIZE as u32);
        ptr.add(Offset::QueuePfn.scale32()).write_volatile(pfn);
        status |= StatusField::DriverOk.val32();
        ptr.add(Offset::Status.scale32()).write_volatile(status);

        let gpu =GPU::new(queue as *mut Queue, ptr, idx);
        let device = DEVICE.as_mut().unwrap();
        device.push(gpu);

        true
    }
}

pub fn pending(pin : usize) {
	// Here we need to check the used ring and then free the resources
	// given by the descriptor id.
	unsafe {
        if let Some(gpu) = &mut DEVICE{
            for dev in gpu{
                if dev.device_idx == pin{
                    let queue = &*dev.queue;
                    while dev.used_idx as u16 != queue.used.idx {
                        let ref elem = queue.used.ring
                            [dev.used_idx as usize % VIRTIO_RING_SIZE];
                        let mut idx = elem.id as usize;
                        while queue.desc[idx].flags & VIRTIO_DESC_F_NEXT != 0 {
                            free(queue.desc[idx].addr as *mut u8);
                            idx = queue.desc[idx].next as usize;
                        }
                        if (*(queue.desc[idx].addr as *const ControllHeader)).ctype != ControllType::RespOkNoData{
                            println!("GPU Err {:?}", (*(queue.desc[idx].addr as *const ControllHeader)).ctype);
                        }
                        free(queue.desc[idx].addr as *mut u8);
                        dev.used_idx = dev.used_idx.wrapping_add(1);
                    }
                    dev.mutex.unlock();
                }
            }
        }
        
	}
}

/// ## 绘图api
/// 

pub fn draw_rect_override(device_idx : usize, rect : Rect, color_buffer : *mut Pixel){
    unsafe {
        if let Some(gpus) = &mut DEVICE{
            let gpu = &mut gpus[device_idx];
            gpu.draw_rect_override(rect, color_buffer);            
        }
    }
}

pub fn draw_rect_blend(device_idx : usize, rect : Rect, color_buffer : *mut Pixel){
    unsafe {
        if let Some(gpus) = &mut DEVICE{
            let gpu = &mut gpus[device_idx];
            gpu.draw_rect_blend(rect, color_buffer);            
        }
    }
}

pub fn invalid(){
    unsafe {
        if let Some(gpus) = &mut DEVICE{
            for gpu in gpus {
                let rect = Rect{x1:0, y1:0, x2:gpu.width as u32, y2:gpu.height as u32};
                gpu.transfer(rect, gpu.display_idx);
                gpu.flush(rect, gpu.display_idx);
                gpu.run();
                gpu.valid = true;
            }
        }
    }
}

pub fn reset(device_idx : usize){
    unsafe {
        if let Some(gpus) = &mut DEVICE{
            let gpu = &mut gpus[device_idx];
            gpu.reset();
        }
    }
}

/// ## IMPL
/// 

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

///////

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Rect{
    pub x1 : u32,
    pub y1 : u32,
    pub x2 : u32,
    pub y2 : u32,
}
#[allow(dead_code)]
#[repr(C)]
#[derive(Debug)]
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

use crate::{memory::{allocator::{alloc, free}, page::{PAGE_SIZE, alloc_kernel_page}}, sync::Mutex, uart};
use core::{cmp::min, mem::size_of};
use alloc::{prelude::v1::*};
use device::{Descriptor, VIRTIO_DESC_F_NEXT, VIRTIO_DESC_F_WRITE};
use crate::libs::graphic::Pixel;
use super::device::{self, Offset, Queue, StatusField, VIRTIO_RING_SIZE};

