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



### 模块重构与接口化

内核将以接口形式提供各种功能，在模块的 `mod.rs` 文件中提供操作接口，具体实现交由子模块完成。对于子模块中可替代部分，使用trait 与泛型进行接口化，便于替换。

#### 任务系统重构（已完成）

任务系统现使用任务管理器统御所有功能模块。功能模块抽象为调度器与任务池，任务管理器接受实现 `TaskPoolBasicOp`、`SchedulerOp` 的泛型。任务系统基本信息变为程序信息（ProgramInfo）、执行信息（ExecutionInfo）。

```rust
pub struct TaskManager<T1 : SchedulerOp, T2 : TaskPoolBasicOp> {
    scheduler : T1,
    task_pool : T2,
    mutex : Mutex,
}
```

在目前的实现中，任务池分为进程、线程。进程管理程序的全局信息，线程负责调度、执行。

#### 内存管理重构（已完成）

使用内存管理器处理页面与堆内存，页面与堆内存提供初始化、申请、释放接口即可

```rust
pub struct MemoryManager<T1 : PageOp, T2 : MemoryOp<T1>> {
    pub page : T1,
    pub memory : T2,
    memory_lock : Mutex,
    page_lock : Mutex,
}
```

目前页面管理使用简单的数组管理，分配按照最先适配原则。堆内存使用类似 SLAB 的链表管理。

#### 设备管理（可完善）

设备管理进行了封装处理，主要为 `VirtQueue`、`VirtHeader` 的封装，同时所有操作由设备管理器执行，所有设备由设备管理器控制。

```rust
pub struct Device {
    pub block_device : Vec<Block>,
    pub gpu_device : Vec<GPU>,
    pub input_device : Vec<InputDevice>,
    dtype : [DeviceType;9],
}
```

### 完善标准输入输出（待完成）

目前输入部分 `src/virtio/input/` 通过设备中断处理，获取设备输入（鼠标、键盘），保存在全局输入缓冲 `src/virtio/input/input_buffer` 中。可以作为标准输入，但是这个设计比较简单，同时缺乏标准输出。现在需要查找已有资料，设计新的标准输入输出系统。

要求：

* 尽可能模块化、接口化
* 最好顺便为标准输入、输出添加相关系统调用

## 欢迎持续补充......