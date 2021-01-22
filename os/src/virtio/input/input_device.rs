#![allow(dead_code)]
//! # Input Device
//! 处理输入设备接口
//! 除了 select 和 subsel 都不能动，size 为 0 则没有信息
//! 2020年12月25日 zg

use alloc::{prelude::v1::*};
use alloc::collections::VecDeque;

const VIRTIO_INPUT_CFG_UNSET : usize = 0x00;
const VIRTIO_INPUT_CFG_ID_NAME : usize = 0x01; // subsel 0，返回名字字符串形式
const VIRTIO_INPUT_CFG_ID_SERIAL : usize = 0x02; // subsel 0，返回 Selrial Number，字符串形式
const VIRTIO_INPUT_CFG_ID_DEVIDS : usize = 0x03; // subsel 0，返回 ID，ids
const VIRTIO_INPUT_CFG_PROP_BITS : usize = 0x10; // subsel 0，返回 Property，在 bitmap 中
const VIRTIO_INPUT_CFG_EV_BITS : usize = 0x11; // subsel 确定是哪个事件
const VIRTIO_INPUT_CFG_ABS_INFO : usize = 0x12; // subsel 确定绝对坐标

#[repr(C)]
pub struct InputABSInfo{
    min : u32,
    max : u32,
    fuzz : u32,
    flat : u32,
    res : u32,
}
#[repr(C)]
struct InputDevids{
    bustype : u16,
    vendor : u16,
    product : u16,
    version : u16,
}
#[repr(C)]
struct InputConfig{
    select : u8,
    subsel : u8,
    size : u8,
    reserved : [u8;5],
    u : T,
}
#[repr(C)]
union T{
    s : [u8;128],
    bitmap : [u8;128],
    info : InputABSInfo,
    ids : InputDevids,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct InputEvent{
    etype : EventType,
    code : u16,
    value : u32,
}

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum EventType {
    Syn = 0x00,
    Key = 0x01,
    Rel = 0x02,
    Abs = 0x03,
    Msc = 0x04,
    Sw = 0x05,
    Led = 0x11,
    Snd = 0x12,
    Rep = 0x14,
    Ff = 0x15,
    Pwr = 0x16,
    FfStatus = 0x17,
    Max = 0x1f,
}
pub struct InputDevice{
	pin_idx : usize,
	queue_idx : usize,
    event_used_idx : usize,
    status_used_idx : usize,
    buffer : *mut InputEvent,
	pub event_queue : *mut Queue, // 0
	pub status_queue : *mut Queue, // 1
	ptr : *mut u32,
}
const EVENT_SIZE: usize = size_of::<InputEvent>();
const EVENT_BUFFER_ELEMENTS : usize = 64;
pub static mut ABS_EVENTS: Option<VecDeque<InputEvent>> = None;
pub static mut KEY_EVENTS: Option<VecDeque<InputEvent>> = None;

impl InputDevice {
    /// ## 新建输入设备管理
    /// 负责初始化状态队列、事件队列
    pub fn new(pin : usize, ptr : *mut u32) ->Self {
        let n = (size_of::<Queue>() + PAGE_SIZE - 1) / PAGE_SIZE;
        let eq = alloc_kernel_page(n) as *mut Queue;
        let sq = alloc_kernel_page(n) as *mut Queue;
        unsafe {
            ptr.add(Offset::QueueSel.scale32()).write_volatile(0);
            ptr.add(Offset::GuestPageSize.scale32()).write_volatile(PAGE_SIZE as u32);
            ptr.add(Offset::QueuePfn.scale32()).write_volatile(eq as u32 / PAGE_SIZE as u32);
            ptr.add(Offset::QueueSel.scale32()).write_volatile(1);
            ptr.add(Offset::QueuePfn.scale32()).write_volatile(sq as u32 / PAGE_SIZE as u32);
        }
        Self{
            pin_idx : pin,
            queue_idx : 0,
            event_used_idx : 0,
            status_used_idx : 0,
            event_queue : eq,
            status_queue : sq,
            buffer : alloc(EVENT_SIZE * EVENT_BUFFER_ELEMENTS, true) as *mut InputEvent,
            ptr : ptr,
        }
    }
    pub unsafe fn fill_event(&mut self, buffer_idx : usize) {
        let desc = Descriptor{
            addr : self.buffer.add(buffer_idx % EVENT_BUFFER_ELEMENTS) as u64,
            len : EVENT_SIZE as u32,
            flags : VIRTIO_DESC_F_WRITE,
            next : 0,
        };
        let head = self.queue_idx as u16;
        (*self.event_queue).desc[self.queue_idx] = desc;
        self.queue_idx = (self.queue_idx + 1) % VIRTIO_RING_SIZE;
        (*self.event_queue).avail.ring[(*self.event_queue).avail.idx as usize % VIRTIO_RING_SIZE] = head;
        (*self.event_queue).avail.idx = (*self.event_queue).avail.idx.wrapping_add(1);
    }
    pub fn pending(&mut self){
        unsafe {
			let ref queue = *self.event_queue;
            while self.event_used_idx as u16 != queue.used.idx{
                let ref elem = queue.used.ring[self.event_used_idx % VIRTIO_RING_SIZE];
                let ref desc = queue.desc[elem.id as usize];
                let event = (desc.addr as *const InputEvent).as_ref().unwrap();
                self.fill_event(elem.id as usize);
                self.event_used_idx = self.event_used_idx.wrapping_add(1);
				// println!("code: 0x{:x} value: 0x{:x} event {}", event.code, event.value, event.etype as u16);
                match event.etype {
                    EventType::Abs => {
                        let mut ev = ABS_EVENTS.take().unwrap();
                        ev.push_back(*event);
                        if ev.len() >= 2{
                            let e = ev.pop_front().unwrap();
                            let x = e.value as f32 / 32767.0;
                            let e = ev.pop_front().unwrap();
                            let y = e.value as f32 / 32767.0;
                            ev.pop_front();
                            add_mouse_position(Point{x:x,y:y});
                        }
                        ABS_EVENTS.replace(ev);
                    },
                    EventType::Key => {
                        let mut ev = KEY_EVENTS.take().unwrap();
                        ev.push_back(*event);
                        if ev.len() >= 1{
                            let e = ev.pop_front().unwrap();
                            if e.value == 1{
                                add_key_press(e.code);
                            }
                            else if e.value == 0{
                                add_key_release(e.code);
                            }
                        }
                        KEY_EVENTS.replace(ev);	
                    },
                    EventType::Rel => {
                    }
                    _ => {
                    }
                }
            }

            let ref queue = *self.status_queue;
            while self.status_used_idx as u16 != queue.used.idx {
                let ref elem = queue.used.ring[self.status_used_idx as usize % VIRTIO_RING_SIZE];
                let ref desc = queue.desc[elem.id as usize];
                let event = (desc.addr as *const InputEvent).as_ref().unwrap();
                println!("Type = {:x}, Code = {:x}, Value = {:x}", event.etype as u8, event.code, event.value);
                self.status_used_idx = self.status_used_idx.wrapping_add(1);
            }
        }
    }
}

pub static mut INPUT_DEVICE : Option<Vec<InputDevice>> = None;
pub fn init(){
    unsafe {
        INPUT_DEVICE = Some(Vec::<InputDevice>::new());
        println!("before 100");
		ABS_EVENTS = Some(VecDeque::with_capacity(100));
        println!("before 10");
		KEY_EVENTS = Some(VecDeque::with_capacity(10));
    }
}

pub fn setup_input_device(ptr: *mut u32, idx : usize) -> bool {
	unsafe {
		ptr.add(Offset::Status.scale32()).write_volatile(0);
		let mut status_bits = StatusField::Acknowledge.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		status_bits |= StatusField::DriverOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		let mut host_features = ptr.add(Offset::HostFeatures.scale32()).read_volatile();
		host_features &= !(1 << VIRTIO_F_RING_EVENT_IDX);
		ptr.add(Offset::GuestFeatures.scale32()).write_volatile(host_features);
		status_bits |= StatusField::FeaturesOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);
		let status_ok = ptr.add(Offset::Status.scale32()).read_volatile();
		if StatusField::FeaturesOk.val32() == status_ok {
			print!("features fail...");
			ptr.add(Offset::Status.scale32()).write_volatile(StatusField::Failed.val32());
			return false;
		}
		
		let qnmax = ptr.add(Offset::QueueNumMax.scale32()).read_volatile();
		ptr.add(Offset::QueueNum.scale32()).write_volatile(VIRTIO_RING_SIZE as u32);
		if VIRTIO_RING_SIZE as u32 > qnmax {
			print!("queue size fail...");
			return false;
		}
		
		if let Some(inputs) = &mut INPUT_DEVICE{
			let mut input = InputDevice::new(idx, ptr);
			for i in 0..EVENT_BUFFER_ELEMENTS{
				input.fill_event(i);
			}
			inputs.push(input);
		}

		status_bits |= StatusField::DriverOk.val32();
		ptr.add(Offset::Status.scale32()).write_volatile(status_bits);

		true
	}
}

pub fn interrupt_handler(pin : usize){
    unsafe {
        if let Some(input) = &mut INPUT_DEVICE{
            for device in input{
                if device.pin_idx == pin {
                    device.pending();
                    break;
                }
            }
        }
    }
}

use core::mem::size_of;

use crate::virtio::{device::{Descriptor, Offset, Queue, StatusField, VIRTIO_DESC_F_WRITE, VIRTIO_F_RING_EVENT_IDX, VIRTIO_RING_SIZE}};
use crate::{memory::{global_allocator::{alloc}},
    page::{PAGE_SIZE, alloc_kernel_page}, uart};

use super::input_buffer::{Point, add_key_press, add_key_release, add_mouse_position};

