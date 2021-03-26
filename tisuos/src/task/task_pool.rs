//! # 任务池
//! 
//! 2021年3月23日 zg


use crate::{interrupt::trap::Environment, sync::Mutex};

use super::{task_info::ExecutionInfo, process::Process, task_manager::{TaskPoolOp, TaskState}, thread::Thread};
use alloc::{prelude::v1::*};
use alloc::collections::BTreeMap;

pub struct TaskPool {
    pub process : BTreeMap<usize, Process>,
    pub thread : BTreeMap<usize, Thread>,
    pub thread_lock : Mutex,
    pub process_lock : Mutex,
}

impl TaskPool {
    pub fn new()->Self {
        Self{
            process : BTreeMap::new(),
            thread : BTreeMap::new(),
            thread_lock : Mutex::new(),
            process_lock : Mutex::new(),
        }
    }
}

impl TaskPoolOp for TaskPool {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize> {
        let mut p = Process::new(is_kernel).unwrap();
        let t = Thread::new(func, &p).unwrap();
        let tid = t.tid;
        self.process_lock.lock();
        p.tid.push(t.tid);
        self.process.insert(p.pid, p);
        self.process_lock.unlock();
        self.thread_lock.lock();
        self.thread.insert(t.tid, t);
        self.thread_lock.unlock();

        Some(tid)
    }

    fn set_task_exec<F>(&mut self, id : usize, f:F)->Result<(), ()>where F:Fn(&mut ExecutionInfo) {
        self.thread_lock.lock();
        if let Some(th) = self.thread.get_mut(&id) {
            let mut info = th.get_exec_info();
            f(&mut info);
            th.set_exec_info(&info);
        }
        self.thread_lock.unlock();
        Err(())
    }

    fn get_task_exec(&mut self, id : usize)->Option<ExecutionInfo> {
        self.thread_lock.lock();
        for (tid, th) in self.thread.iter() {
            if *tid == id {
                self.thread_lock.unlock();
                return Some(th.get_exec_info());
            }
        }
        self.thread_lock.unlock();
        None
    }

    fn get_task_prog(&mut self, id : usize)->Option<super::task_info::ProgramInfo> {
        self.thread_lock.lock();
        for (tid, th) in self.thread.iter() {
            if *tid == id {
                self.process_lock.lock();
                for (pid, p) in self.process.iter() {
                    if *pid == th.pid {
                        self.thread_lock.unlock();
                        self.process_lock.unlock();
                        return Some(p.get_prog_info());
                    }
                }
                break;
            }
        }
        self.thread_lock.unlock();
        self.process_lock.unlock();
        None
    }

    fn remove_task(&mut self, id : usize)->Result<(), ()> {
        self.thread_lock.lock();
        self.thread.remove(&id);
        self.thread_lock.unlock();
        Err(())
    }

    fn remove_program(&mut self, id : usize)->Result<(), ()> {
        self.thread_lock.lock();
        let mut v = vec![];
        for (tid, t) in self.thread.iter() {
            if t.pid == id {
                v.push(*tid);
            }
        }

        for idx in v {
            self.thread.remove(&idx);
        }
        self.thread_lock.unlock();
        self.process_lock.lock();
        self.process.remove(&id);
        self.process_lock.unlock();
        Err(())
    }

    fn select<F>(&mut self, f : F)->Option<Vec<usize>> where F : Fn(&ExecutionInfo)->bool {
        self.thread_lock.lock();
        let mut res = vec![];
        for (tid, t) in self.thread.iter() {
            if f(&t.get_exec_info()) {
                res.push(*tid);
            }
        }
        self.thread_lock.unlock();
        if res.len() > 0 {
            Some(res)
        }
        else{
            None
        }
    }

    fn find<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool {self.thread_lock.lock();
        for (tid, t) in self.thread.iter() {
            if f(&t.get_exec_info()) {
                self.thread_lock.unlock();
                return Some(*tid);
            }
        }
        self.thread_lock.unlock();
        None
    }

    fn read<F>(&mut self, mut f:F) where F:FnMut(&ExecutionInfo) {
        for (_, t) in self.thread.iter() {
            f(&t.get_exec_info());
        }
    }

    fn fork(&mut self, env : &Environment)->Option<usize> {
        let src_th = self.select_thread(|info| {
            info.state == TaskState::Running && info.env.hartid == env.hartid
        }).unwrap();
        src_th.save(env);
        let th = Thread::copy(src_th).unwrap();
        let id = th.tid;
        self.thread_lock.lock();
        self.thread.insert(id, th);
        self.thread_lock.unlock();
        Some(id)
    }
}

impl TaskPool {
    fn select_thread<F>(&mut self, f : F)->Option<&mut Thread> where F:Fn(&ExecutionInfo)->bool {
        self.thread_lock.lock();
        for (_, t) in self.thread.iter_mut() {
            if f(&t.get_exec_info()) {
                self.thread_lock.unlock();
                return Some(t);
            }
        }
        self.thread_lock.unlock();
        None
    }
}

