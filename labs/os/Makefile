TARGET      := riscv64imac-unknown-none-elf
MODE        := debug
KERNEL_FILE := target/$(TARGET)/$(MODE)/os
BIN_FILE    := target/$(TARGET)/$(MODE)/kernel.bin

USER_DIR    := ../user
USER_BUILD  := $(USER_DIR)/build
IMG_FILE    := $(USER_BUILD)/disk.img
TEST_IMG    := ./disk.img

OBJDUMP     := rust-objdump --arch-name=riscv64
OBJCOPY     := rust-objcopy --binary-architecture=riscv64

.PHONY: doc kernel build clean qemu run

# 默认 build 为输出二进制文件
build: $(BIN_FILE) 

# 通过 Rust 文件中的注释生成 os 的文档
doc:
	@cargo doc --document-private-items

# 编译 kernel
kernel:
	@cargo build

# 生成 kernel 的二进制文件
$(BIN_FILE): kernel
	@$(OBJCOPY) $(KERNEL_FILE) --strip-all -O binary $@

# 查看反汇编结果
asm:
	@$(OBJDUMP) -d $(KERNEL_FILE) | less

# 清理编译出的文件
clean:
	@cargo clean

# 运行 QEMU
qemu: build
	@qemu-system-riscv64 \
			-machine virt \
			-nographic \
			-bios default \
			-device loader,file=$(BIN_FILE),addr=0x80200000 \
			-drive file=$(TEST_IMG),format=qcow2,id=sfs \
			-device virtio-blk-device,drive=sfs  
# 一键运行
run: build qemu

# 一键 gdb
debug: build
		mate-terminal -e "gdb-multiarch -ex 'file $(KERNEL_FILE)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'b handle_interrupt' -ex 'c'" &
		qemu-system-riscv64 -machine virt -nographic -bios default -device loader,file=$(BIN_FILE),addr=0x80200000 \
		-drive file=$(TEST_IMG),format=qcow2,id=sfs -device virtio-blk-device,drive=sfs -s -S
