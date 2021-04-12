#####
## BUILD
#####
CC=riscv64-unknown-elf-gcc
CFLAGS=-Wall -Wextra -pedantic -Wextra -O0 -g
CFLAGS+=-static -ffreestanding -nostdlib -fno-rtti -fno-exceptions
CFLAGS+=-march=rv64gc -mabi=lp64
INCLUDES=
LINKER_SCRIPT=-Tsrc/lds/virt.lds
TYPE=debug
RUST_TARGET=./target/riscv64gc-unknown-none-elf/$(TYPE)
KERNEL_ELF=$(RUST_TARGET)/tisuos
LIBS=-L$(RUST_TARGET)
SOURCES_ASM=$(wildcard src/asm/*.S)
LIB=-l tisuos -lgcc
OUT=os.elf

#####
## QEMU
#####
QEMU=qemu-system-riscv64
MACH=virt
CPU=rv64
CPUS=4
MEM=512M
DISK=hd.dsk
DRIVER = -device virtio-gpu-device -device\
 virtio-net-device -device virtio-tablet-device\
 -device virtio-keyboard-device -drive if=none,format=raw,file=$(DISK),id=foo\
 -device virtio-blk-device,scsi=off,drive=foo


build:
	cargo build
# $(CC) $(CFLAGS) $(LINKER_SCRIPT) $(INCLUDES) -o $(OUT) $(SOURCES_ASM) $(LIBS) $(LIB)
	
run: build
	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) $(DRIVER) -nographic -serial mon:stdio -bios none -kernel $(KERNEL_ELF)


.PHONY: clean
clean:
	cargo clean
	rm -f $(OUT)
