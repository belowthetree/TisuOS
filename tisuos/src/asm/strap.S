.align 4
.global s_trap_vector
s_trap_vector:
	csrw	satp, zero
	sfence.vma
	csrrw	t6, sscratch, t6
	.set 	i, 0
	.rept	31
		save_gp	%i
		.set	i, i+1
	.endr
	
	mv		t5, t6
	csrr	t6, sscratch
	save_gp 31, t5

	csrw	sscratch, t5

	.set 	i, 0
	.rept	32
		save_fp	%i, t5
		.set	i, i+1
	.endr
	
	csrr	t0, sepc
	sd		t0, 520(t5)
	csrr	t1, satp
	sd		t1, 512(t5)
	li		a0, 3
	ecall
	sd		a0, 528(t5)

	# Get ready to go into Rust (trap.rs)
	# We don't want to write into the user's stack or whomever
	# messed with us here.
	mv		a2, a0
	mv		a0, t5
	csrr	a1, scause
	csrr	a3, sscratch
	mv		a4, sp
	csrr	a5, stval
	la		sp, _trap_stack_end2
	li		t1, 0x40000
	mul		t1, t1, a2
	sub		sp, sp, t1
	call	s_trap

	sfence.vma
	csrw	sepc, a0

	# Now load the trap frame back into t6
	csrr	t6, sscratch

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

	csrrw	t5, stvec, t5
	csrrw	t6, sscratch, t6
	ld		t5, 512(t6)
	csrrw	t6, sscratch, t6
	csrw	satp, t5
	la		t5, s_trap_vector
	csrrw	t5, stvec, t5

	sret

# a0: *mut Environment
.global switch_user_process
switch_user_process:
	mv		t6, a0
	ld		t0, 512(t6)
	csrrw	t6, sscratch, t6
	sd		t0, 512(t6)
	csrrw	t6, sscratch, t6
	# csrw 	satp, t0
	ld		t1, 520(t6)
	csrw 	sepc, t1

	li		t3, 1 << 5 | 1 << 13
	csrw	sstatus, t3
	
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
	csrrw	t6, sscratch, t6
	csrrw	t5, stvec, t5
	ld		t5, 512(t6)
	csrrw	t6, sscratch, t6
	csrw	satp, t5
	la		t5, s_trap_vector
	csrrw	t5, stvec, t5

	sfence.vma

	sret

.global switch_kernel_process
switch_kernel_process:
	sfence.vma
	mv		t6, a0
	ld		t0, 512(t6)
	csrw 	satp, t0
	ld		t1, 520(t6)
	csrw 	sepc, t1

	la		t2, s_trap_vector
	csrw	stvec, t2
	li		t3, 1 << 5 | 1 << 13 | 1 << 8 | 1 << 7
	csrw	sstatus, t3
	
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

	sret
