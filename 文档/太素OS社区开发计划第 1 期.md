# 太素OS社区开发计划第 1 期

交流QQ群：961871659

## 开发目的

本次开发分为两个目标：

* 整理内核代码，分析各部分的模块化、接口化编程的情况、可能性。然后在群里讨论决定好重构措施
* 目前决定好将内存管理部分 `src/memory`、任务系统部分 `src/task` 做成接口化

所以这次的目标主要是确定一个好的接口化、模块化方式

## 具体工作

接口话意味着将对应模块对外接口功能做成 `trait`，方便将其中对应的算法替换。如内存管理部分目前使用一个类似 SLAB 的算法进行分配管理，可以将其功能概括为如下 trait：

```rust
pub trait MemoryManage {
    fn alloc_kernel_memory(size : usize)->Option<*mut u8>;
    fn alloc_user_memory(size : usize)->Option<*mut u8>;
    fn free_kernel_memory(addr : *mut u8);
    fn free_user_memory(addr : *mut u8);
}
```

这是目前一个简单的接口化、模块化工作。具体如何实现可以先行讨论，确定一个不错的方案。



**所以目前需要各位针对太素OS提出一个可行的接口化、模块化建议**



### 任务系统进一步细分

任务系统分为进程与线程，其中的调度工作主要在 `thread.rs` 中。目前打算实现一个调度器 `Schedule`，将调度工作交由调度器完成，由调度器操控进程与线程，并且将进程、线程的操作接口全部收归为结构体的方法。