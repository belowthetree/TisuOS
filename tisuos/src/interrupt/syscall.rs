//! # System call
//! 系统调用转到这里处理
//! 2020年12月18日 zg

/// 进程退出自动调用
const PROGRAM_EXIT  : usize = 60;
/// 线程退出自动调用
const THREAD_EXIT   : usize = 61;
const FORK          : usize = 57;
const BRANCH        : usize = 7;
/// 调试用输出
const PRINT_TASK    : usize = 5;
/// 接收路径创建进程
const EXEC          : usize = 4;
const MALLOC        : usize = 8;
/// 等待某个任务结束，@tid:usize
const WAIT          : usize = 9;
/// 睡眠调用者，定时唤醒，@time:usize
const SET_TIMER     : usize = 10;
const FREE          : usize = 11;
/// 打开文件，@path:str;@flag:usize->id:usize
const OPEN          : usize = 12;
/// @id:usize->len:usize
const READ          : usize = 13;
/// 获取文件信息，@id:usize->*const FileInfo
const FILE_INFO     : usize = 14;
/// @id:usize->len:usize
const WRITE         : usize = 15;
const DRAW_RECT     : usize = 16;
const GET_TIME      : usize = 17;
const GET_KEY_PRESS     : usize = 18;
const GET_KEY_RELEASE   : usize = 19;
const GET_MOUSE_SCROLL  : usize = 20;
const GET_MOUSE_POS     : usize = 21;
/// 获取目录信息，@id:usize->*const DirectoryInfo
const DIRECTORY_INFO    : usize = 22;
/// 关闭文件，打开后必须关闭，目前进程结束会自动关闭所有文件，@id:usize
const CLOSE             : usize = 23;
/// @id:usize
const KILL              : usize = 24;
/// 关闭时钟、软件中断
const CLOSE_TIMER       : usize = 25;
/// 开启时钟、软件中断
const OPEN_TIMER        : usize = 26;
const SHUTDOWN          : usize = 27;
/// 使目标线程睡眠，@id:usize
const SLEEP             : usize = 28;
/// 唤醒目标线程，@id:usize
const WAKE              : usize = 29;
/// 暂停当前线程，等待同进程内其它线程
const JOIN              : usize = 30;
const GET_TID           : usize = 31;
const NEXT              : usize = 32;

static mut CLOSE_CNT : [usize;4] = [0;4];

pub enum SyscallResult {
    Schedule(usize),
    Normal(usize),
}

pub fn handler(env : &mut Environment)->SyscallResult {
    let mut rt = SyscallResult::Normal(0);
    let num = env.regs[Register::A0.val()];
    match num {
        1 => {
            println!("syscall test");
        }
        2 => {
            println!("syscall test2");
        }
        3 => {
            rt = SyscallResult::Normal(env.hartid);
        }
        NEXT => {
            rt = SyscallResult::Schedule(0);
        }
        GET_TID => {
            let mgr = get_task_mgr().unwrap();
            let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
            rt = SyscallResult::Normal(exec.tid);
        }
        JOIN => {
            let mgr = get_task_mgr().unwrap();
            let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
            mgr.join(exec.tid, env);
            rt = SyscallResult::Schedule(0);
        }
        WAKE => {
            let mgr = get_task_mgr().unwrap();
            mgr.wake_task(env.a1());
        }
        SLEEP => {
            let mgr = get_task_mgr().unwrap();
            let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
            mgr.sleep_task(exec.tid, env).unwrap();
            rt = SyscallResult::Schedule(0);
        }
        SHUTDOWN => {
            unsafe {
                const VIRT_TEST: *mut u32 = 0x10_0000 as *mut u32;
                VIRT_TEST.write_volatile(0x5555);
            }
        }
        OPEN_TIMER => {
            // println!("open");
            unsafe {
                CLOSE_CNT[env.hartid] -= 1;
                if CLOSE_CNT[env.hartid] == 0 {
                    asm!("
                        li t0, 1 << 7 | 1 << 3
                        csrs mie, t0
                    ")
                }
            }
        }
        CLOSE_TIMER => {
            // println!("close");
            unsafe {
                CLOSE_CNT[env.hartid] += 1;
                asm!("
                    li t0, 1 << 7 | 1 << 3
                    csrc mie, t0
                ")
            }
        }
        KILL => {
            kill(env);
            rt = SyscallResult::Schedule(0);
        }
        CLOSE => {
            close(env);
        }
        DIRECTORY_INFO => {
            rt = SyscallResult::Normal(directory_info(env));
        }
        GET_MOUSE_POS => {
            rt = SyscallResult::Normal(get_mouse_x());
            env.regs[Register::A1.val()] = get_mouse_y();
        }
        GET_MOUSE_SCROLL => {
            rt = SyscallResult::Normal(get_scroll());
        }
        GET_KEY_RELEASE => {
            if let Some(key) = get_key_release() {
                rt = SyscallResult::Normal(key);
            }
        }
        GET_KEY_PRESS => {
            if let Some(key) = get_key_press() {
                rt = SyscallResult::Normal(key);
            }
        }
        GET_TIME => {
            rt = SyscallResult::Normal(timer::get_micro_time());
        }
        DRAW_RECT => {
            draw_rect(env);
        }
        WRITE => {
            let len = env.regs[Register::A3.val()];
            if len != 0 {
                let mgr = get_task_mgr().unwrap();
                let (exec,_) = mgr.get_current_task(env.hartid).unwrap();
                let id = env.regs[Register::A1.val()];
                let addr = env.regs[Register::A2.val()];
                let ptr = mgr.virt_to_phy(exec.tid, addr) as *const u8;
                let data = unsafe{& *(slice_from_raw_parts(ptr, len))};
                if let Ok(len) = write(exec.pid, id, data) {
                    rt = SyscallResult::Normal(len);
                }
                else {
                    println!("read fail")
                }
            }
        }
        READ => {
            let len = env.regs[Register::A3.val()];
            if len != 0 {
                let mgr = get_task_mgr().unwrap();
                let (exec,_) = mgr.get_current_task(env.hartid).unwrap();
                let id = env.regs[Register::A1.val()];
                let addr = env.regs[Register::A2.val()];
                let ptr = mgr.virt_to_phy(exec.tid, addr) as *mut u8;
                let data = unsafe{&mut *(slice_from_raw_parts_mut(ptr, len))};
                if let Ok(len) = read(exec.pid, id, data) {
                    rt = SyscallResult::Normal(len);
                }
                else {
                    println!("task {} read fail", exec.tid);
                }
            }
        }
        FILE_INFO => {
            rt = SyscallResult::Normal(file_info(env));
        }
        OPEN => {
            rt = SyscallResult::Normal(open(env) as usize);
        }
        FREE => {
            let mgr = get_task_mgr().unwrap();
            let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
            mgr.free_heap(env.regs[Register::A1.val()], exec.tid);
        }
        WAIT => {
            let mgr = get_task_mgr().unwrap();
            mgr.wait_task(env, env.regs[Register::A1.val()]);
            rt = SyscallResult::Schedule(0);
        }
        MALLOC => {
            let mgr = get_task_mgr().unwrap();
            let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
            let t = mgr.alloc_heap(env.regs[Register::A1.val()], exec.tid);
            rt = SyscallResult::Normal(t.0);
        }
        SET_TIMER => {
            let time = env.regs[Register::A1.val()];
            get_task_mgr().unwrap().sleep_timer(env, time);
            rt = SyscallResult::Schedule(0);
        }
        EXEC => {
            rt = SyscallResult::Normal(exec(env));
        }
        PRINT_TASK => {
            get_task_mgr().unwrap().print();
        }
        BRANCH => {
            rt = SyscallResult::Normal(branch(env));
        }
        FORK => {
            rt = SyscallResult::Normal(fork(env));
        }
        PROGRAM_EXIT => {
            // println!("delete process");
            let mgr = get_task_mgr().unwrap();
            let (e,_) = mgr.get_current_task(env.hartid).unwrap();
            mgr.program_exit(e.tid);
            rt = SyscallResult::Schedule(0);
        }
        THREAD_EXIT => {
            // println!("delete thread");
            let mgr = get_task_mgr().unwrap();
            let (e,_) = mgr.get_current_task(env.hartid).unwrap();
            mgr.task_exit(e.tid);
            rt = SyscallResult::Schedule(0);
        }
        _ => {}
    }
    rt
}

fn kill(env : &Environment) {
    let id = env.a1();
    let mgr = get_task_mgr().unwrap();
    mgr.kill_task(id);
}

fn close(env : &Environment) {
    let mgr = get_task_mgr().unwrap();
    let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
    let id = env.a1();
    if let Some(sys) = search_system(id) {
        if let Some(file) = sys.file(id) {
            if file.is_own(exec.tid) {
                for (i, owner) in file.state.owner.iter().enumerate() {
                    if exec.pid == *owner {
                        mgr.release_file(exec.tid, id);
                        file.state.owner.remove(i);
                        break;
                    }
                }
            }
        }
    }
}

fn directory_info(env : &Environment)->usize {
    let mgr = get_task_mgr().unwrap();
    let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
    let ptr = mgr.virt_to_phy(exec.tid, env.a1());
    let all_path = from_ptr(ptr as *mut char);
    let idx = all_path.find("/").unwrap();
    let (id, path) = all_path.split_at(idx);
    let id = convert_to_usize(&id.to_string());
    if let Some(sys) = get_system(id) {
        let flag = env.a2();
        if let Ok(dir) = sys.enter(path.to_string().clone()) {
            let mut dir_num = 0;
            let mut file_num = 0;
            for item in dir.item.iter() {
                if item.is_dir() {
                    dir_num += 1;
                }
                else {
                    file_num += 1;
                }
            }
            let t = 15 * size_of::<char>();
            let size = size_of::<DirectoryInfo>() + dir_num * t + file_num * t;
            let (va, pa) = mgr.alloc_heap(size, exec.tid);
            let ptr = pa as *mut u8 as *mut DirectoryInfo;
            unsafe {(*ptr).replace(&dir, dir_num, file_num, 15)}
            let ptr = (pa + size_of::<DirectoryInfo>()) as *mut char;
            let mut idx = 0;
            for file in dir.item.iter() {
                if file.is_file() {
                    let ptr = unsafe {ptr.add(idx)};
                    write_str(ptr, &file.name, t);
                    idx += 15;
                }
            }
            for dir in dir.item.iter() {
                if dir.is_dir() {
                    let ptr = unsafe {ptr.add(idx)};
                    write_str(ptr, &dir.name, t);
                    idx += 15;
                }
            }
            va
        }
        else {
            println!("file info open err path {}, flag {}", path, flag);
            0
        }
    }
    else {
        println!("file info device err {}", idx);
        0
    }
}

fn draw_rect(env : &Environment) {
    if !gpu_support() {
        return;
    }
    let rect = Rect{
        x1:env.a1() as u32,
        y1:env.a2() as u32,
        x2:env.a3() as u32,
        y2:env.a4() as u32};
    let mgr = get_task_mgr().unwrap();
    let (exec,_) = mgr.get_current_task(env.hartid).unwrap();
    let data = mgr.virt_to_phy(exec.tid, env.a5()) as *const Pixel;
    let len = (rect.x2 - rect.x1) * (rect.y2 - rect.y1);
    let buffer = unsafe {&*(slice_from_raw_parts(data, len as usize))};
    get_device().draw_rect_override(0, rect, buffer);
    invalid();
}

fn file_info(env : &Environment)->usize {
    let mgr = get_task_mgr().unwrap();
    let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
    let ptr = mgr.virt_to_phy(exec.tid, env.a1());
    let all_path = from_ptr(ptr as *mut char);
    let idx = all_path.find("/").unwrap();
    let (id, path) = all_path.split_at(idx);
    let id = convert_to_usize(&id.to_string());
    if let Some(sys) = get_system(id) {
        let flag = env.a2();
        if let Ok(file) = sys.get_file(path.to_string().clone()) {
            let file_info = FileInfo::new(
                file.id,
                file.device_id,
                file.start_idx,
                file.state.flag.val(),
                file.size
            );
            let (p, ptr) = mgr.alloc_heap(size_of::<FileInfo>(), exec.tid);
            let ptr = ptr as *mut FileInfo;
            unsafe {
                ptr.write_volatile(file_info);
            }
            p as usize
        }
        else {
            println!("file info open err path {}, flag {}", path, flag);
            0
        }
    }
    else {
        println!("file info device err {}", idx);
        0
    }
}

fn open(env : &Environment)->isize {
    let mgr = get_task_mgr().unwrap();
    let (exec, _) = mgr.get_current_task(env.hartid).unwrap();
    let all_path = from_ptr(mgr.virt_to_phy(exec.tid, env.a1()) as *mut char);
    let idx = all_path.find("/").unwrap();
    let (id, path) = all_path.split_at(idx);
    let id = convert_to_usize(&id.to_string());
    if let Some(sys) = get_system(id) {
        let flag = env.a2();
        if let Ok(file) = sys.open(path.to_string().clone(), FileFlag::from(flag).unwrap()) {
            file.own(exec.pid);
            mgr.push_file(exec.tid, file.id);
            file.id as isize
        }
        else {
            println!("open err path {}, flag {}", path, flag);
           -1 as isize
        }
    }
    else {
        println!("device err {}", idx);
        -1
    }
}

fn fork(env : &Environment)->usize {
    get_task_mgr().unwrap().fork_task(env)
}

fn branch(env : &Environment)->usize {
    let mgr = get_task_mgr().unwrap();
    mgr.branch(env).unwrap()
}

fn exec(env : &Environment)->usize {
    let ptr = env.regs[Register::A1.val()] as *mut char;
    let len = env.regs[Register::A2.val()];
    let path = unsafe {&*(slice_from_raw_parts(ptr, len))};
    let path = char_to_str(path);
    let is_kernel = env.regs[Register::A3.val()] != 0;
    let idx = path.find("/").unwrap();
    let (id, p) = path.split_at(idx);
    let id = convert_to_usize(&id.to_string());
    let sys = get_system(id).unwrap();
    let file = sys.open(p.to_string(), tisu_fs::FileFlag::Read).unwrap().clone();
    let data = Block::<u8>::new(file.size);
    sys.read(file.id, data.to_array(0, file.size)).unwrap();
    let elf = data.type_as::<ELF>();
    if !elf.is_elf() {
        return 0;
    }
    let mut elf = ElfManager::new(elf);
    let mgr = get_task_mgr().unwrap();
    let mut program = ProgramArea::new(elf.entry(), is_kernel);
    program.push_elf(&mut elf);
    let task_id = mgr.create_task(program, env).unwrap();
    mgr.wake_task(task_id);
    task_id
}


use core::{mem::size_of, ptr::{slice_from_raw_parts, slice_from_raw_parts_mut}};


use tisu_driver::{Pixel, Rect};
use tisu_fs::{FileFlag, SystemOp};
use crate::{filesystem::{DirectoryInfo, FileInfo, elf::{ELF, ElfManager}, get_system, search_system, syscall_io::{read, write}}, libs::{str::{char_to_str, convert_to_usize, from_ptr, write_str}}, memory::{ProgramArea, block::Block}, virtio::{device::{get_device, gpu_support, invalid},
    input_buffer::{get_key_press, get_key_release, get_mouse_x, get_mouse_y, get_scroll}}};
use crate::task::get_task_mgr;
use super::{environment::{Environment, Register}, timer};
use crate::alloc::string::ToString;
use core::arch::asm;