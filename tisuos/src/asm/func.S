

.global process_exit
process_exit:
	li	a0, 60
	ecall
	ret

.global thread_exit
thread_exit:
	li	a0, 61
	ecall
	ret

.global write_mscratch
write_mscratch:
	csrw	mscratch, a0
	ret

.global read_mhartid
read_mhartid:
	csrr	a0, mhartid
	ret