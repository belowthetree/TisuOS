TARGET := riscv64gc-unknown-none-elf
MODE := release
LIB := libuser_lib
SRC_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
SRC := $(wildcard $(SRC_DIR)/*.rs)
ELF := $(patsubst $(SRC_DIR)/%.rs, $(TARGET_DIR)/%, $(SRC))
APP_DIR := apps
IMAGE = img


binary:
	cargo build --release

copy:binary
	@$(foreach file, $(ELF), mv $(file) $(patsubst $(TARGET_DIR)/%, $(APP_DIR)/%.elf, $(file));)

img: FORCE
#	cargo install image-maker
	image-maker $(APP_DIR) $(IMAGE)
	cp $(IMAGE) ../tisuos/

build: copy img

.PHONY: FORCE binary img copy 
FORCE: