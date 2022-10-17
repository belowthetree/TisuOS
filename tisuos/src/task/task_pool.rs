//! # 任务池
//! 
//! 2021年3月23日 zg


use crate::{interrupt::environment::Environment, memory::ProgramArea};
use tisu_sync::ContentMutex;
use super::{process::Process, require::{TaskComplexOp, TaskPoolBasicOp, TaskPoolOp, TaskResourceOp, TaskScheduleOp}, task_info::{ExecutionInfo, TaskState}, thread::Thread};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

pub struct TaskPool {
    process : ContentMutex<BTreeMap<usize, Process>>,
    thread : ContentMutex<BTreeMap<usize, Thread>>,
    waiting_list : ContentMutex<BTreeMap<usize, Vec<usize>>>,
    wait_time_list : ContentMutex<BTreeMap<usize, usize>>,
    time_list : ContentMutex<Vec<usize>>,
}

impl TaskPool {
    pub fn new()->Self {
        Self{
            process : ContentMutex::new(BTreeMap::new(), true),
            thread : ContentMutex::new(BTreeMap::new(), true),
            waiting_list : ContentMutex::new(BTreeMap::new(), true),
            wait_time_list : ContentMutex::new(BTreeMap::new(), true),
            time_list : ContentMutex::new(Vec::new(), true),

        }
    }
}

impl TaskPoolOp for TaskPool{}

/// 为了防止死锁，线程必须先于进程上锁
impl TaskPoolBasicOp for TaskPool {
    fn create(&mut self, program : ProgramArea, env : &Environment)->Option<usize> {
        let mut p = Process::new(program).unwrap();
        let t = Thread::new(&p, env).unwrap();
        let tid = t.info.tid;
        p.tid.push(t.info.tid);
        self.thread.lock().insert(t.info.tid, t);
        self.process.lock().insert(p.info.pid, p);

        Some(tid)
    }

    fn fork(&mut self, env : &Environment)->Option<usize> {
        let id = self.find(|info| {
            info.state == TaskState::Running && info.env.hartid == env.hartid
        }).unwrap();
        let mut thread = self.thread.lock();
        let src_th = thread.get_mut(&id).unwrap();
        src_th.save(env);
        let th = Thread::fork(src_th).unwrap();
        let id = th.info.tid;
        let mut process = self.process.lock();
        let p = process.get_mut(&th.info.pid).unwrap();
        p.tid.push(id);
        thread.insert(id, th);
        Some(id)
    }

    fn branch(&mut self, env : &Environment)->Option<usize> {
        let id = self.find(|info| {
            info.state == TaskState::Running && info.env.hartid == env.hartid
        }).unwrap();
        let mut thread = self.thread.lock();
        let src_th = (*thread).get_mut(&id).unwrap();
        src_th.save(env);
        let th = Thread::branch(src_th).unwrap();
        let id = th.info.tid;
        let pid = th.info.pid;
        (*thread).insert(id, th);
        let mut process = self.process.lock();
        process.get_mut(&pid).unwrap().tid.push(id);
        Some(id)
    }

    fn get_task_exec(&self, id : usize)->Option<ExecutionInfo> {
        let thread = self.thread.lock();
        if let Some(th) = thread.get(&id) {
            Some(th.get_exec_info())
        }
        else {
            None
        }
    }

    fn get_task_prog(&self, id : usize)->Option<super::task_info::ProgramInfo> {
        if let Some(t) = self.thread.lock().get(&id) {
            let id = t.info.pid;
            let rt = self.process.lock().get(&id).unwrap().get_prog_info();
            Some(rt)
        }
        else {
            None
        }
    }

    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool {
        let thread = self.thread.lock();
        let mut res = vec![];
        for (tid, t) in (*thread).iter() {
            if f(&t.info) {
                res.push(*tid);
            }
        }
        if res.len() > 0 {
            Some(res)
        }
        else{
            None
        }
    }

    fn find<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool {
        let thread = self.thread.lock();
        for (tid, t) in (*thread).iter() {
            if f(&t.info) {
                return Some(*tid);
            }
        }
        None
    }

    fn operation_all<F>(&mut self, mut f:F) where F:FnMut(&mut ExecutionInfo) {
        let mut thread = self.thread.lock();
        for (_, t) in (*thread).iter_mut() {
            f(&mut t.info);
        }
    }

    fn operation_once<F>(&mut self, mut f:F) where F:FnMut(&ExecutionInfo)->bool {
        let mut thread = self.thread.lock();
        for (_, th) in (*thread).iter_mut() {
            if f(&mut th.info) {
                break;
            }
        }
    }

    fn set_task_exec<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo) {
        let mut thread = self.thread.lock();
        if let Some(th) = (*thread).get_mut(&id) {
            f(&mut th.info);
            Ok(())
        }
        else {
            Err(())
        }
    }

    fn send_task_msg(&mut self, id : usize, _msg : &crate::memory::block::Block<u8>) {
        let process = self.process.lock();
        let p = process.get(&id);
        if let Some(_p) = p {
            // p.heap_list
        }
        else {}
    }

    fn set_task_prog<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut super::task_info::ProgramInfo) {
        let mut process = self.process.lock();
        let p = process.get_mut(&id);
        if let Some(p) = p {
            f(&mut p.info);
            Ok(())
        }
        else {
            Err(())
        }
    }

    fn remove_task(&mut self, id : usize)->Result<(), ()> {
        let mut thread = self.thread.lock();
        let pid = thread.get(&id).unwrap().info.pid;
        thread.remove(&id);
        let list = self.waiting_list.lock();
        if let Some(waiter) = list.get(&id) {
            for id in waiter {
                thread.get_mut(id).unwrap().wake();
            }
        }
        let mut process = self.process.lock();
        let p = process.get_mut(&pid).unwrap();
        for (idx, tid) in p.tid.iter().enumerate() {
            if *tid == id {
                p.tid.remove(idx);
                break;
            }
        }
        Ok(())
    }

    fn remove_program(&mut self, id : usize)->Result<(), ()> {
        let mut thread = self.thread.lock();
        let pid = thread.get(&id).unwrap().info.pid;
        let mut v = vec![];
        for (tid, t) in (*thread).iter() {
            if t.info.pid == pid {
                v.push(*tid);
            }
        }
        let list = self.waiting_list.lock();

        for tid in v {
            (*thread).remove(&tid);
            if let Some(waiter) = list.get(&tid) {
                for id in waiter {
                    thread.get_mut(id).unwrap().wake();
                }
            }
        }
        let mut process = self.process.lock();
        process.remove(&pid).unwrap();
        Ok(())
    }

    fn print(&self) {
        let process = self.process.lock();
        for (_, p) in process.iter() {
            println!("program #{}# {:?}, threads: ", p.info.pid, p.info.state);
            for tid in p.tid.iter() {
                let info = self.get_task_exec(*tid).unwrap();
                println!("#{}# {:?} priority {}", info.tid, info.state, info.priority);
            }
        }
    }
}

impl TaskComplexOp for TaskPool {
    fn alloc_heap(&mut self, size : usize, id : usize)->(usize, usize) {
        let thread = self.thread.lock();
        let pid = thread.get(&id).unwrap().info.pid;
        let mut process = self.process.lock();
        let p = process.get_mut(&pid).unwrap();
        p.alloc_heap(size)
    }

    fn free_heap(&mut self, addr : usize, id : usize) {
        let thread = self.thread.lock();
        let pid = thread.get(&id).unwrap().info.pid;
        let mut process = self.process.lock();
        let p = process.get_mut(&pid).unwrap();
        p.free_heap(addr);
    }

    fn virt_to_phy(&self, id:usize, va:usize)->usize {
        let thread = self.thread.lock();
        let t = thread.get(&id).unwrap();
        let pid = t.info.pid;
        let process = self.process.lock();
        let p = process.get(&pid).unwrap();
        if p.contain(va) {
            p.virt_to_phy(va)
        }
        else {
            t.virt_to_phy(va)
        }
    }

    fn wait_task(&mut self, waiter: usize, target: usize) {
        let mut thread = self.thread.lock();
        let wait_thread = thread.get_mut(&waiter).unwrap();
        wait_thread.sleep();
        let wait_id = wait_thread.info.tid;
        if let Some(target_th) = thread.get(&target) {
            let mut list = self.waiting_list.lock();
            if let Some(waiter) = list.get_mut(&target_th.info.tid) {
                waiter.push(wait_id);
            }
            else {
                let mut v = Vec::new();
                v.push(wait_id);
                list.insert(target_th.info.tid, v);
            }
        }
        else {
            let wait_thread = thread.get_mut(&waiter).unwrap();
            wait_thread.wake();
        }
    }

    fn check_timer(&mut self, time : usize) {
        let mut thread = self.thread.lock();
        let mut time_list = self.time_list.lock();
        let mut wait_list = self.wait_time_list.lock();
        let mut cnt = 0;
        for tm in time_list.iter() {
            if *tm <= time {
                let id = wait_list.get(&tm).unwrap();
                if let Some(th) = thread.get_mut(&id) {
                    th.info.trigger_time = 0;
                    th.info.state = TaskState::Waiting;
                    cnt += 1;
                }
            }
        }
        for _ in 0..cnt {
            let time = time_list.remove(0);
            wait_list.remove(&time).unwrap();
        }
    }

    fn set_timer(&mut self, id : usize, time : usize) {
        let mut thread = self.thread.lock();
        let th = thread.get_mut(&id).unwrap();
        th.info.trigger_time = time;
        th.info.state = TaskState::Sleeping;
        let mut time_list = self.time_list.lock();
        let mut wait_list = self.wait_time_list.lock();
        wait_list.insert(time, th.info.tid);
        time_list.push(time);
        time_list.sort();
    }

    fn expand_stack(&mut self, id : usize)->Result<(),()> {
        let mut thread = self.thread.lock();
        let th = thread.get_mut(&id).unwrap();
        let process = self.process.lock();
        let p = process.get(&th.info.pid).unwrap();
        th.expand_stack(&p.info.satp)
    }

    fn join(&mut self, id : usize) {
        let mut thread = self.thread.lock();
        let t = thread.get_mut(&id).unwrap();
        let mut process = self.process.lock();
        let p = process.get_mut(&t.info.pid).unwrap();
        p.join_num += 1;
        if p.join_num >= p.tid.len() {
            for tid in p.tid.iter() {
                thread.get_mut(tid).unwrap().info.state = TaskState::Waiting;
            }
            p.join_num = 0;
        }
        else {
            t.info.state = TaskState::Sleeping;
        }
    }
}

impl TaskResourceOp for TaskPool {
    fn push_file(&mut self, task_id : usize, file_id:usize) {
        let pid = self.thread.lock().get(&task_id).unwrap().info.pid;
        let mut process = self.process.lock();
        process.get_mut(&pid).unwrap().push_file(file_id);
    }

    fn release_file(&mut self, task_id : usize, file_id:usize) {
        let pid = self.thread.lock().get(&task_id).unwrap().info.pid;
        let mut process = self.process.lock();
        process.get_mut(&pid).unwrap().release_file(file_id);
    }
}

impl TaskScheduleOp for TaskPool {
    fn set_priority(&mut self, id:usize, priority : usize) {
        self.thread.lock().get_mut(&id).unwrap().info.priority = priority;
    }
}
