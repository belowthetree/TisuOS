# VirtIO说明

VirtIO 的使用可以分成两个部分来理解：Queue、Header

设备通过内存映射（MMIO）将一系列寄存器位映射到内存中，在QEMU的实现中，设备分布在 0x1000_1000~0x1000_8000中，每个间隔 0x1000 一共八个设备。当读取其中一个设备映射的起始地址时（比如读取0x1000_1000），本质上是在读取下方的`Header`结构。所以可以理解为，设备将自己的 Header 放到了内存中，其中包含了一些信息，同时可以通过向 Header 写入数据来通知设备。

Queue 则是作为一个交流的数据结构，Queue中有三个队列：存储命令的 desc 队列；系统用的 avail 队列；设备用的 used 队列。

## Header

```rust
pub struct VirtHeader {
    magicvalue : u32,
    version : u32,
    deviceid : u32, // 标志设备类型
    vendorid : u32,
    host_features : u32, // 设备支持的特性
    host_features_sel : u32,
    rev0 : [u32;2],
    guest_features : u32, // 系统支持的特性
    guest_features_sel : u32,
    guest_page_size : u32, // 系统使用的页面大小
    rev1 : u32,
    queue_sel : u32, // 选择哪个队列
    queue_num_max : u32, // 队列最大长度
    queue_num : u32,
    queue_align : u32,
    queue_pfn : u32, // 队列所在页面的下标（从 0 开始）
    rev2 : [u32;3],
    queue_notify : u32, // 通知设备处理命令
    rev3 : [u32;3],
    interrupt_status : u32,
    interrupt_ack : u32, // 标志设备是否产生中断
    rev4 : [u32;2],
    status : u32, // 设备状态，由下方的 StatusField 描述
}

pub enum StatusField {
	Acknowledge = 1, // 通知设备它已经被发现
	Driver = 2,
	Failed = 128,
	FeaturesOk = 8, // 通知设备，特性已经设置完毕，如果此为在通知后仍然是 0，说明设置失败
	DriverOk = 4, // 通知系统设置完毕
	DeviceNeedsReset = 64,
}
```

Header 负责初始化设备。以块设备为例，一般流程如下（需要结合下方源码）：

* 首先设置本系统支持的特性，set_feature，块设备不需要只读属性，所以该位置 0
* 然后设置 Queue 中的队列长度，直接写入
* 设置本系统使用的页面大小，直接写入
* 告诉设备，本系统为它准备的 Queue 的地址，需要先写入 queue_sel 告知设置的是哪个Queue，因为有些设备需要设置多个 Queue
* 通知它系统驱动设置完毕

```rust
fn init_block() {
    let num = (size_of::<VirtQueue>() + PAGE_SIZE - 1) / PAGE_SIZE;
    let queue = alloc_kernel_page(num).unwrap() as *mut VirtQueue;
    let header = unsafe {&mut *header};
    header.set_feature(!(1 << VIRTIO_BLK_F_RO)).unwrap();
    header.set_ring_size(VIRTIO_RING_SIZE as u32).unwrap();
    header.set_page_size(PAGE_SIZE as u32);
    header.set_pfn(0, queue);
    header.driver_ok();
}
impl VirtHeader {
    pub fn set_feature(&mut self, guest_feat : u32)->Result<(), ()> {
        self.status = 0;
        self.status = StatusField::Acknowledge.val32();
        self.status |= StatusField::DriverOk.val32();
        self.guest_features = self.host_features & guest_feat;
        self.status |= StatusField::FeaturesOk.val32();
        let status_ok = self.status;
        if status_ok & StatusField::FeaturesOk.val32() == 0 {
            println!("Set up block device fail");
            return Err(());
        }
        Ok(())
    }

    pub fn set_ring_size(&mut self, size : u32)->Result<(), ()> {
        if self.queue_num_max < size {
            Err(())
        }
        else {
            self.queue_num = size;
            Ok(())
        }
    }

    pub fn set_pfn(&mut self, sel : u32, addr : *mut VirtQueue) {
        self.queue_sel = sel;
        self.queue_pfn = addr as u32 / PAGE_SIZE as u32;
    }

    pub fn set_page_size(&mut self, size : u32) {
        self.guest_page_size = size;
    }

    pub fn driver_ok(&mut self) {
        self.status = StatusField::DriverOk.val32();
    }
}
```



## Queue

```rust
pub struct VirtQueue {
	pub desc:  [Descriptor; VIRTIO_RING_SIZE],
	pub avail: Available,
	pub padding0: [u8; PAGE_SIZE - size_of::<Descriptor>() * VIRTIO_RING_SIZE - size_of::<Available>()],
	pub used:     Used,
}
pub struct Descriptor {
	pub addr:  u64,
	pub len:   u32,
	pub flags: u16,
	pub next:  u16,
}
pub enum DescFlag {
	Next = 1,
	Write = 2,
}
pub struct Available {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [u16; VIRTIO_RING_SIZE],
	pub event: u16,
}
pub struct Used {
	pub flags: u16,
	pub idx:   u16,
	pub ring:  [UsedElem; VIRTIO_RING_SIZE],
	pub event: u16,
}
pub struct UsedElem {
	pub id:  u32,
	pub len: u32,
}
```

Queue 中队列功能不同：

* desc 储存 Descriptor 即描述符，描述符各字段作用如下：

  * addr 指向命令的地址，至于命令是什么格式，取决于设备类型。
  * len 标志命令结构的长度
  * flags 太素用到了两种，Next、Write，Next 标志这个命令还有剩余部分存储在下一个描述符，Write 标志设备可以写入此描述符
  * next 在 flags 的 Next 位为 1 的情况下指出下一个描述符的编号

* avail 是系统用来告诉设备 desc 的写入情况的

  * flags 没用上，具体可以查阅 virtio 手册

  * idx 指出下一个将要写入的 ring 的下标
  * ring 存储命令在 desc 中的下标，比如 desc[0] 被写入，那么 avail.ring[avail.idx] 应该为 0，接着 avail.idx 应该加一
  * event 未知

* used 是设备用来告知系统已经读取了哪些 desc

  * idx 与 avail 一样，指出下一个将要写入的 ring 的下标
  * ring 储存 UsedItem，标志已经读取了的 desc 的下标及长度

可以看出，avail 的功能是一致的，只不过一个是给系统用，一个是给设备用，都是单向写入的。

## 使用流程

```rust
pub fn sync_read(&mut self, buffer : *mut u8, size : u32, offset : usize) {
    let rq = Request::new(buffer, offset, false);
    let header = unsafe {&(*rq).header as *const Header};
    let status = unsafe {&(*rq).status as *const u8};
    let mut flag = DescFlag::Next as u16;
    self.queue.add_avail();
    self.queue.add_desc(header as u64,size_of::<Header>() as u32,flag);
    flag |= DescFlag::Write as u16;
    self.queue.add_desc(buffer as u64, size, flag);
    flag = DescFlag::Write as u16;
    self.queue.add_desc(status as u64, 1, flag);
    self.header.notify();
}
```

以块设备的读取命令为例。

* 首先 add_avail() 将要写入的 desc 下标写入 avail.ring 中
* 将命令的信息放入 Descriptor 中记录，然后放入 desc 队列中。此处用了两个 add_desc，第一个添加了命令本身，第二个添加了缓冲区，因为这两个 Descriptor 是关联的，所以第一个 flag 的 Next 被置位
* 最后通知设备

```rust
impl VirtQueue {
    pub fn add_desc(&mut self, addr : u64, len : u32, flag : u16) {
        let next;
        if flag & DescFlag::Next as u16 == 0 {next = 0}
        else {next = (self.desc_idx + 1) % VIRTIO_RING_SIZE as u16}

        let desc = Descriptor {
            addr: addr,
            len: len,
            flags: flag,
            next: next,
        };
        self.desc[self.desc_idx as usize] = desc;
        self.desc_idx = (self.desc_idx + 1) % VIRTIO_RING_SIZE as u16;
    }

	pub fn add_avail(&mut self) {
		self.avail.ring[self.avail.idx as usize % VIRTIO_RING_SIZE] = self.desc_idx;
		self.avail.idx = self.avail.idx.wrapping_add(1);
	}

	pub fn is_pending(&self)->bool {
		self.used_idx != self.used.idx
	}

	pub fn next_elem(&mut self)->UsedElem {
		let elem = self.used.ring[self.used_idx as usize % VIRTIO_RING_SIZE];
		self.used_idx = self.used_idx.wrapping_add(1);
		elem
	}
}
```

## GPU 使用流程

初始化与块设备一致，但是多出了许多命令格式（块设备只有一种命令用于写入读取）。

* create_resouce_id，用于创建一个缓冲区的 id
* attach，用于绑定创建好的缓冲区 id
* set_scanout，将扫描器 id 与缓冲区的 id 绑定，通常只有一个扫描器
* transfer，传输缓冲区数据至显存
* flush，刷新显存内容到屏幕

```rust
fn add_desc<T>(&mut self, addr1 : u64, ctype : ControllType) {
    let header = ControllHeader::new();
    unsafe {(*header).ctype = ctype;}

    let ref mut q = self.queue;
    q.add_avail();
    q.add_desc(addr1, size_of::<T>() as u32, DescFlag::Next as u16);
    q.add_desc(header as u64, size_of::<ControllHeader>() as u32,
        DescFlag::Write as u16);

}
```

GPU 的命令添加略有不同，如上方所示，添加完特定的命令后需要再添加一个 ControllType 来告诉设备命令的类型，这是因为 GPU 的命令比较多，所以需要额外的一个命令来告知命令类型。