# TisuOS

[![](https://img.shields.io/github/license/belowthetree/TisuOS)](./LICENSE)

**QQ交流群号：961871659**

Tisu OS is a RISCV-based operating system written in Rust and designed primarily for educational purposes, as well as a reference for beginners.

![](图/系统截图.jpg)

gitee：https://gitee.com/belowthetree/tisu-os （include hd image and will be updated firstly）

github：https://github.com/belowthetree/TisuOS （pure code）

## Build

environment：

* wsl
* rust-nightly

just run `make build` in `/tisuos`

## Run

platform: QEMU

* qemu-system-riscv64

`make run`

## Content

### TisuOS

> `/tisuos`

Newest implementation os TisuOS

### document

> `/document`

include development plans and OS-dev document

### os-tutorial

> `/os-tutorial`

OS used in tutorial, is more simple but has some bugs. It can work in most cases.

### tutorial

> `/教程`

only has Chinese version

## kernel module

| name              | content                                                      | state                           |
| ----------------- | ------------------------------------------------------------ | ------------------------------- |
| device            | UART、block device、GPU、keyboard、mouse                     | finished. net device need added |
| interrupt handler | timer、syscall、error handle、software                       | finish basic function           |
| memory manager    | page manager（fisrt fit）、heap（like SLAB algorithm）       | finish                          |
| task system       | process、thread、scheduler                                   | schedule task in turn           |
| filesystem        | support FAT32 、[TianMu](https://github.com/TisuOS/tianmu-fs) format | support basic func              |
| graphic desktop   | similar with dock and window                                 | can explore and interact simply |

