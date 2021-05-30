# 太素 TisuOS

[![](https://img.shields.io/github/license/belowthetree/TisuOS)](./LICENSE)  [![](https://img.shields.io/badge/english-readme-blue)](./README-EN.md)

**QQ交流群号：961871659**

太素OS是一个用 Rust 编写的基于 RISCV 架构的操作系统，主要用于教学目的，为初学者提供参考。

![](图/系统截图.jpg)

gitee 地址：https://gitee.com/belowthetree/tisu-os （包含硬盘镜像，最新推送）

github 地址：https://github.com/belowthetree/TisuOS （纯源码版本，删除了镜像、target 文件夹）

## 编译

环境：

* wsl
* rust-nightly

在 `tisuos` 下 运行 `make build`

**根据热心群友 wjl 提醒，今后版本不使用 gcc 编译**

## 运行：

平台：QEMU

* qemu-system-riscv64

指令：`make run`

## 内容

### TisuOS

> `/tisuos`

太素 OS 的最新实现

### 文档

> `/document`

包括硬件、系统设计结构、开发计划等文档

### OS-tutorial

> `/os-tutorial`

教程中使用的系统，有一些 BUG，但是不影响大部分情况的运行

### 教程

> `/教程`

分章节进行讲解，提供每个章节的源码

## 内核功能模块

| 功能     | 内容                                                         | 状况                             |
| -------- | ------------------------------------------------------------ | -------------------------------- |
| 设备管理 | UART、块设备、GPU、键盘、鼠标                                | 已完成，但网络等其它设备待添加   |
| 中断处理 | 时钟中断、系统调用、错误处理、软件中断                       | 完成基本功能                     |
| 内存管理 | 页表管理（最先适配方式）、堆内存管理（类SLAB算法）           | 已完成                           |
| 任务系统 | 进程、线程、调度器                                           | 完成基本功能，其它调度方法待添加 |
| 文件系统 | 支持 FAT32 、[TianMu](https://github.com/TisuOS/tianmu-fs) 格式 | 完成基本功能                     |
| 图形桌面 | 拥有类似 dock、窗口的功能                                    | 拥有基本浏览交互功能             |

## License

[GPL-v3](./LICENSE)

