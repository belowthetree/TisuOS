//! # 任务池
//! 
//! 2021年3月23日 zg


use core::cmp::Ordering;

use crate::{interrupt::trap::Environment, sync::Mutex};

use super::{info::ExecutionInfo, process::Process, task_manager::{TaskPoolOp},
    thread::Thread};
use alloc::{prelude::v1::*};

pub struct TaskPool {
    pub process : Vec<Process>,
    pub thread : Vec<Thread>,
    pub thread_lock : Mutex,
    pub process_lock : Mutex,
}

impl TaskPool {
    pub fn new()->Self {
        Self{
            process : Vec::<Process>::new(),
            thread : Vec::<Thread>::new(),
            thread_lock : Mutex::new(),
            process_lock : Mutex::new(),
        }
    }
}

impl TaskPoolOp for TaskPool {
    fn create(&mut self, func : usize, is_kernel : bool)->Option<usize> {
        let p = Process::new(func, is_kernel).unwrap();
        let id = p.pid;
        self.thread_lock.lock();
        self.process.push(p);
        self.thread_lock.unlock();
        Some(id)
    }

    fn set_task_exec(&mut self, id : usize, info : &ExecutionInfo)->Result<(), ()> {
        self.thread_lock.lock();
        for th in self.thread.iter_mut() {
            if th.tid == id {
                th.set_exec_info(info);
                self.thread_lock.unlock();
                return Ok(());
            }
        }
        self.thread_lock.unlock();
        Err(())
    }

    fn get_task_exec(&mut self, id : usize)->Option<ExecutionInfo> {
        self.thread_lock.lock();
        for th in self.thread.iter() {
            if th.tid == id {
                self.thread_lock.unlock();
                return Some(th.get_exec_info());
            }
        }
        self.thread_lock.unlock();
        None
    }

    fn get_task_prog(&mut self, id : usize)->Option<super::info::ProgramInfo> {
        self.thread_lock.lock();
        for th in self.thread.iter() {
            if th.tid == id {
                self.process_lock.lock();
                for p in self.process.iter() {
                    if p.pid == th.pid {
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

    fn delete_task(&mut self, id : usize)->Result<(), ()> {
        self.thread_lock.lock();
        for (i, th) in self.thread.iter().enumerate() {
            if th.tid == id {
                self.thread.remove(i);
                self.thread_lock.unlock();
                return Ok(());
            }
        }
        self.thread_lock.unlock();
        Err(())
    }

    fn select<F>(&mut self, f : F)->Option<usize> where F : Fn(&ExecutionInfo)->bool {
        self.thread_lock.lock();
        for t in self.thread.iter() {
            if f(&t.get_exec_info()) {
                self.thread_lock.unlock();
                return Some(t.tid);
            }
        }
        self.thread_lock.unlock();
        None
    }

    fn sort<F>(&mut self, f : F)->usize where F : Fn(&ExecutionInfo, &ExecutionInfo)->Ordering {
        self.thread_lock.lock();
        self.thread.sort_by(|a, b|{
            f(&a.get_exec_info(), &b.get_exec_info())
        });
        let id = self.thread.first().unwrap().tid;
        self.thread_lock.unlock();
        id
    }

    fn fork(&mut self, env : &Environment)->Option<usize> {
        let th = Thread::copy(&env).unwrap();
        let id = th.tid;
        self.thread_lock.lock();
        self.thread.push(th);
        self.thread_lock.unlock();
        Some(id)
    }
}


