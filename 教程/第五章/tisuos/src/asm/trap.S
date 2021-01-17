.option norvc
.altmacro
.set NUM_GP_REGS, 32  # Number of registers per context
.set NUM_FP_REGS, 32
.set REG_SIZE, 8   # Register size (in bytes)
.set MAX_CPUS, 8   # Maximum number of CPUs

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
	.set 	i, 1
	.rept	30
		save_gp	%i
		.set	i, i+1
	.endr
	
	.set 	i, 0
	.rept	32
		save_fp	%i
		.set	i, i+1
	.endr
	
	mv		t5, t6
	csrr	t6, mscratch
	save_gp 31, t5

	csrw	mscratch, t5
	csrr	t0, mepc
	sd		t0, 520(t5)
	csrr	t1, satp
	sd		t1, 512(t5)

	# Get ready to go into Rust (trap.rs)
	# We don't want to write into the user's stack or whomever
	# messed with us here.
	mv		a0, t5
	csrr	a1, mcause
	csrr	a2, mhartid
	csrr	a3, mstatus
	mv		a4, sp
	la		t0, _trap_stack_end
	li		t1, 0x10000
	mul		t1, t1, a2
	sub		sp, t0, t1
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
	.set	i, 1
	.rept	31
		load_gp %i
		.set	i, i+1
	.endr

	mret
