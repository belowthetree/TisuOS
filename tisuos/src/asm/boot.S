# boot.S
# 2020年10月6日 zg

# 禁止压缩指令
.option norvc
.section .text.init
.global _start
_start:

.option push
# 根据手册说明，意思是不允许链接器松弛（？）
.option norelax
	# 据手册说明，GP 寄存器用来访问它周围的内存，看起来可以加速访问呢
	la		gp, _global_pointer
.option pop
	# PMP 配置，默认开启所有地址访问权限
	li		t0, 0xffffffffffffffff
	csrw	pmpaddr0, t0
	li		t0, 0b10001111
	csrw	pmpcfg0, t0
	# 让其它（非0号）硬件线程挂起，跳转至 3
	csrr	t0, mhartid
	csrw	satp, zero
	bnez	t0, 3f
	# 将 BSS 段清零
	la 		a0, _bss_start
	la		a1, _bss_end
	bgeu	a0, a1, 2f
1:	# 清零
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:
	la		sp, _stack_end
	# 先初始化
	li		t0, (0b11 << 13) | (0b11 << 11) | (1 << 7)
	csrw	mstatus, t0
	la		t1, kernel_init
	csrw	mepc, t1
	la		t2, m_trap_vector
	csrw	mtvec, t2
	li		t3, 0xaaa
	csrw	mie, t3
	la		ra, 4f
	mret
3:
	# 不同核心栈区分开
	la		sp, _stack_end
	li		t0, 0x20000
	csrr	a0, mhartid
	mul		t0, t0, a0
	sub		sp, sp, t0

	# 进入 M 模式，开启中断
	li		t0, 0b11 << 11 | (1 << 7) | (1 << 13) | 1 << 1
	csrw	mstatus, t0
	# 机器软件中断，通过核心 0 调用
	li		t3, 0xa
	csrw	mie, t3
	
	csrr	a0, mhartid
	la		t1, kernel_start
	csrw	mepc, t1
	
	la		t2, m_trap_vector
	csrw	mtvec, t2
	la		t2, s_trap_vector
	csrw	stvec, t2
	li		t3, 1 << 1 | 1 << 13 | 1 << 5
	csrw	sstatus, t3
	li		t3, 0x222
	csrw	mideleg, t3
	csrw	sie, t3
	li		t3, 0xe5ff
	csrw	medeleg, t3

	# Whenever our hart is done initializing, we want it to return to the waiting
	# loop, which is just below mret.
	la		ra, 4f
	# We use mret here so that the mstatus register is properly updated.
	mret

4:
	# wfi = wait for interrupt. This is a hint to the harts to shut everything needed
	# down. However, the RISC-V specification allows for wfi to do nothing. Anyway,
	# with QEMU, this will save some CPU!
	wfi
	j		4b


