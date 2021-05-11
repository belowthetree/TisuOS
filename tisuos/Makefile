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
DISK1=hd.dsk
DISK2=img
DEVICE = -device virtio-tablet-device\
 -device virtio-keyboard-device\
 -drive if=none,format=raw,file=$(DISK1),id=foo -device virtio-blk-device,scsi=off,drive=foo\
 -drive if=none,format=raw,file=$(DISK2),id=fo1 -device virtio-blk-device,scsi=off,drive=fo1

NET_DEVICE = -device virtio-net-device
GPU_DEVICE = -device virtio-gpu-device

build: env
	cargo +nightly build --target=riscv64gc-unknown-none-elf
# $(CC) $(CFLAGS) $(LINKER_SCRIPT) $(INCLUDES) -o $(OUT) $(SOURCES_ASM) $(LIBS) $(LIB)

env:
	(rustup target list | grep "riscv64gc-unknown-none-elf (installed)") || rustup target add $(TARGET)
	rustup component add rust-src
	rustup component add llvm-tools-preview

run: build
	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) $(DEVICE) \
	-nographic -serial mon:stdio -bios none -kernel $(KERNEL_ELF)

graphic: build
	$(QEMU) -machine $(MACH) -cpu $(CPU) -smp $(CPUS) -m $(MEM) $(DEVICE) $(GPU_DEVICE) \
	-serial mon:stdio -bios none -kernel $(KERNEL_ELF)

objdump:
	riscv64-unknown-elf-objdump -d target/riscv64gc-unknown-none-elf/debug/tisuos > os.txt

.PHONY: clean
clean:
	cargo clean
