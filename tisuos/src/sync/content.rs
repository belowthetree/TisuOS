//! # 内容锁
//! 此锁免去解锁操作
//! 
//! 2021年4月3日 zg

#![allow(dead_code)]
use core::ops::{Deref, DerefMut};

use super::mutex::Mutex;


pub struct ContentMutex<T> {
    value : T,
    pub mutex : Mutex,
}

impl<T> ContentMutex<T> {
    pub fn new(value : T)->Self {
        Self {
            value : value,
            mutex : Mutex::new(),
        }
    }

    pub fn lock(&mut self)->Content<T> {
        self.mutex.lock();
        Content::new(self)
    }

    pub fn unlock(&mut self) {
        self.mutex.unlock();
    }
}

pub struct Content<'a, T> {
    mutex : &'a mut ContentMutex<T>,
}

impl<'a, T> Content<'a, T> {
    pub fn new(mutex : &'a mut ContentMutex<T>)->Self {
        Self {
            mutex : mutex,
        }
    }
}

impl<T> Deref for Content<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.mutex.value
    }
}

impl<T> DerefMut for Content<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mutex.value
    }
}

impl<'a, T> Drop for Content<'a, T>{
    fn drop(&mut self) {
        self.mutex.unlock()
    }
}