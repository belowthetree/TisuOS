# trap.S
# Trap handler and global context
# 借用 Stephen Marz 的设计
# 2020年12月
.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set REG_SIZE, 8   # Register size (in bytes)

# Use macros for saving and restoring multiple registers
.macro save_gp i, basereg=t6
	sd	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro load_gp i, basereg=t6
	ld	x\i, ((\i)*REG_SIZE)(\basereg)
.endm
.macro save_fp i, basereg=t6
	fsd	f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm
.macro load_fp i, basereg=t6
	fld	f\i, ((NUM_GP_REGS+(\i))*REG_SIZE)(\basereg)
.endm

.section .text
.global m_trap_vector
# This must be aligned by 4 since the last two bits
# of the mtvec register do not contribute to the address
# of this vector.
.align 4
m_trap_vector:
	csrrw	t6, mscratch, t6
	.set 	i, 0
	.rept	31
		save_gp	%i
		.set	i, i+1
	.endr
	
	mv		t5, t6
	csrr	t6, mscratch
	save_gp 31, t5

	csrw	mscratch, t5

	.set 	i, 0
	.rept	32
		save_fp	%i, t5
		.set	i, i+1
	.endr
	
	csrr	t0, mepc
	sd		t0, 520(t5)
	csrr	t1, satp
	sd		t1, 512(t5)
	csrr	t2, mhartid
	sd		t2, 528(t5)

	# Get ready to go into Rust (trap.rs)
	# We don't want to write into the user's stack or whomever
	# messed with us here.
	mv		a0, t5
	csrr	a1, mcause
	csrr	a2, mhartid
	csrr	a3, mstatus
	mv		a4, sp
	csrr	a5, mtval
	la		sp, _trap_stack_end
	li		t1, 0x20000
	mul		t1, t1, a2
	sub		sp, sp, t1
	call	m_trap
	# When we get here, we've returned from m_trap, restore registers
	# and return.
	# m_trap will return the return address via a0.

	csrw	mepc, a0

	# Now load the trap frame back into t6
	csrr	t6, mscratch

	.set	i, 0
	.rept	32
		load_fp %i
		.set i, i+1
	.endr
	# Restore all GP registers
	.set	i, 0
	.rept	32
		load_gp %i
		.set	i, i+1
	.endr

	mret

.global waiting
waiting:
	wfi
	j waiting

.global make_syscall
make_syscall:
	ecall
	ret

# a0: *mut Environment
.global switch_user_process
switch_user_process:
	mv		t6, a0
	ld		t0, 512(t6)
	csrw 	satp, t0
	ld		t1, 520(t6)
	csrw 	mepc, t1

	# li		t1, 0xaaa
	# csrw	mie, t1
	la		t2, m_trap_vector
	csrw	mtvec, t2
	li		t3, 1 << 4 | 1 << 13
	csrw	mstatus, t3
	
	.set	i, 0
	.rept	32
		load_fp %i
		.set i, i+1
	.endr
	
	# Restore all GP registers
	.set	i, 1
	.rept	31
		load_gp %i
		.set	i, i+1
	.endr

	mret

.global switch_kernel_process
switch_kernel_process:
	mv		t6, a0
	ld		t0, 512(t6)
	csrw 	satp, t0
	ld		t1, 520(t6)
	csrw 	mepc, t1

	# li		t1, 0xaaa
	# csrw	mie, t1
	la		t2, m_trap_vector
	csrw	mtvec, t2
	li		t3, 1 << 5 | 1 << 13 | 1 << 11
	csrw	mstatus, t3
	
	.set	i, 0
	.rept	32
		load_fp %i
		.set i, i+1
	.endr
	
	# Restore all GP registers
	.set	i, 1
	.rept	31
		load_gp %i
		.set	i, i+1
	.endr

	mret

test:
	li a0, 1
	ecall
1:
	wfi
	j 1b

