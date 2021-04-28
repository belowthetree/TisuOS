//! # 任务池
//! 
//! 2021年3月23日 zg


use crate::{interrupt::trap::Environment};
use tisu_sync::ContentMutex;
use super::{task_info::{ExecutionInfo, TaskState}, process::Process, task_manager::{TaskPoolBasicOp},
    thread::Thread};
use alloc::{prelude::v1::*};
use alloc::collections::BTreeMap;

pub struct TaskPool {
    pub process : ContentMutex<BTreeMap<usize, Process>>,
    pub thread : ContentMutex<BTreeMap<usize, Thread>>,
}

impl TaskPool {
    pub fn new()->Self {
        Self{
            process : ContentMutex::new(BTreeMap::new()),
            thread : ContentMutex::new(BTreeMap::new()),
        }
    }
}

impl TaskPoolBasicOp for TaskPool {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize> {
        let mut p = Process::new(is_kernel).unwrap();
        let t = Thread::new(func, &p).unwrap();
        let tid = t.tid;
        p.tid.push(t.tid);
        self.process.lock().insert(p.info.pid, p);
        self.thread.lock().insert(t.tid, t);

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
        let id = th.tid;
        let mut process = self.process.lock();
        let p = process.get_mut(&th.pid).unwrap();
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
        let id = th.tid;
        (*thread).insert(id, th);
        Some(id)
    }

    fn get_task_exec(&self, id : usize)->Option<ExecutionInfo> {
        let thread = self.thread.lock();
        for (tid, th) in (*thread).iter() {
            if *tid == id {
                return Some(th.get_exec_info());
            }
        }
        None
    }

    fn get_task_prog(&self, id : usize)->Option<super::task_info::ProgramInfo> {
        let id = self.thread.lock().get(&id).unwrap().pid;
        let rt = self.process.lock().get(&id).unwrap().get_prog_info();
        Some(rt)
    }

    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool {
        let thread = self.thread.lock();
        let mut res = vec![];
        for (tid, t) in (*thread).iter() {
            if f(&t.get_exec_info()) {
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
            if f(&t.get_exec_info()) {
                return Some(*tid);
            }
        }
        None
    }

    fn operation_all<F>(&mut self, mut f:F) where F:FnMut(&ExecutionInfo) {
        let thread = self.thread.lock();
        for (_, t) in (*thread).iter() {
            f(&t.get_exec_info());
        }
    }

    fn operation_once<F>(&mut self, mut f:F) where F:FnMut(&ExecutionInfo)->bool {
        let thread = self.thread.lock();
        for (_, th) in (*thread).iter() {
            if f(&th.get_exec_info()) {
                break;
            }
        }
    }

    fn set_task_exec<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo) {
        let mut thread = self.thread.lock();
        if let Some(th) = (*thread).get_mut(&id) {
            let mut info = th.get_exec_info();
            f(&mut info);
            th.set_exec_info(&info);
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
            let mut info = p.get_prog_info();
            f(&mut info);
            p.set_prog_info(info);
            Ok(())
        }
        else {
            Err(())
        }
    }

    fn remove_task(&mut self, id : usize)->Result<(), ()> {
        self.thread.lock().remove(&id);
        Ok(())
    }

    fn remove_program(&mut self, id : usize)->Result<(), ()> {
        let mut thread = self.thread.lock();
        let mut v = vec![];
        for (tid, t) in (*thread).iter() {
            if t.pid == id {
                v.push(*tid);
            }
        }

        for idx in v {
            (*thread).remove(&idx);
        }
        let mut process = self.process.lock();
        (*process).remove(&id);
        Ok(())
    }

    fn print(&self) {
        let process = self.process.lock();
        for (_, p) in process.iter() {
            println!("program #{}# {:?}, threads: ", p.info.pid, p.info.state);
            for tid in p.tid.iter() {
                let info = self.get_task_exec(*tid).unwrap();
                println!("#{}# {:?} ", info.tid, info.state);
            }
        }
    }
}

