# Building
OS := $(shell cat ./Cargo.toml | grep -oP 'name = \s*"\K[^"]+')
TARGET := riscv64gc-unknown-none-elf
MODE := release
LOG ?= INFO

FS_IMG := ../user/target/$(TARGET)/release/fs.img
# FS_IMG := ../easy-fs-fuse/fs.img

LOG_FILE ?= kernel.log
KERNEL_ELF := target/$(TARGET)/$(MODE)/$(OS)
KERNEL_BIN := $(KERNEL_ELF).bin


KERNEL_TEST_ELF := $(shell find target/$(TARGET)/debug/deps \
						   		-name '$(OS)-*' \
								! -name '*.d' \
								! -name '*-bin' \
								| head -n 1)
KERNEL_TEST_BIN := target/$(TARGET)/debug/deps/$(OS)-bin

KERNEL_ELF_DEBUG := target/$(TARGET)/debug/$(OS)
KERNEL_BIN_DEBUG := $(KERNEL_ELF_DEBUG).bin

DISASM_TMP = /tmp/asm


# BOARD
# Assign BORAD to K210 by `make BOAED=K210`
BOARD ?= qemu
SBI ?= rustsbi

QEMU_DIR := ../tools/qemu-7.0.0/build
QEMU := $(QEMU_DIR)/qemu-system-riscv64
# QEMU := qemu-system-riscv64
GDB := gdb-multiarch
# GDB := rust-gdb

BOOTLOADER := ../bootloader/$(SBI)-$(BOARD).bin
BOOTLOADER_SIZE_HEX := 0x20000
BOOTLOADER_SIZE := $(shell printf "%d\n" $(BOOTLOADER_SIZE_HEX))

# compiling

ifeq ($(MODE), release)
	MODE_ARG := --release
endif

ifeq ($(BOARD), qemu)
	FEATURES := --features board_qemu
else ifeq ($(BOARD), k210)
	FEATURES := --features board_k210
endif

LINKER_SCRIPT_TEMPLATE = ../scripts/template.linker.ld
LINKER_SCRIPT = $(subst template.,,$(LINKER_SCRIPT_TEMPLATE))

# KERNEL ENTRY
QEMU_ENTRY := 0x80200000
K210_ENTRY := 0x80020000
ifeq ($(BOARD), qemu)
	KERNEL_ENTRY_PA := $(QEMU_ENTRY)
else ifeq ($(BOARD), k210)
	KERNEL_ENTRY_PA := $(K210_ENTRY)
endif

# Running in K210
K210-SERIALPORT := /dev/ttyUSB0
KFLASH := ../tools/kflash.py/kflash.py
KFLASH_SRC := https://gitee.com/peiguodong/kflash.py.git

# Binutils
OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# Disassembly
DISASM ?= -x

build: $(KERNEL_BIN)

# kernel: $(KERNEL_BIN)
# 	@echo Platform: $(BOARD)
# 	@cp src/linker_$(BOARD).ld src/linker.ld
# 	@cargo build $(MODE_ARG)
# 	@rm src/linker.ld

kernel:

	@cd ../user && make build
	@echo Platform: $(BOARD)
	@echo $(LINKER_SCRIPT_TEMPLATE)
	@echo $(LINKER_SCRIPT)
	@echo $(MODE)
	@echo $(LOG)
	@sed 's/#BASE_ADDRESS/$(KERNEL_ENTRY_PA)/' src/$(LINKER_SCRIPT_TEMPLATE) > src/$(LINKER_SCRIPT)
	@LOG=$(LOG) cargo build $(MODE_ARG) $(FEATURES)
	@rm src/$(LINKER_SCRIPT)

$(KERNEL_BIN): kernel
	@$(OBJCOPY) $(KERNEL_ELF) --strip-all -O binary $@


.PHONY: run test clean gdb packfs\
		kernel build disasm debug \
		
run: run-inner

run-inner: build packfs
ifeq ($(BOARD),qemu)
	$(QEMU) \
		-s \
		-machine virt \
		-nographic \
		-bios  $(BOOTLOADER)\
		-device loader,file=$(KERNEL_BIN),addr=$(KERNEL_ENTRY_PA)\
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
		
		2>&1 | tee -a $(LOG_FILE)
else
	(which $(KFLASH) || (cd .. && git clone $(KFLASH_SRC) && mv kflash.py tools))
	@cp $(BOOTLOADER) $(BOOTLOADER).copy
	@dd if=$(KERNEL_BIN) of=$(BOOTLOADER).copy\
		bs=$(BOOTLOADER_SIZE) seek=1
	@mv $(BOOTLOADER).copy $(KERNEL_BIN)
	@sudo chmod 777 $(K210-SERIALPORT)
	python3 $(KFLASH) -p $(K210-SERIALPORT) -b 1500000 $(KERNEL_BIN)
	python3 -m serial.tools.miniterm \
			--eol LF \
			--dtr 0 \
			--rts 0 \
			--filter direct $(K210-SERIALPORT) 115200
endif

debug: build
	@tmux new-session -d \
		"$(QEMU) -machine virt -nographic -bios $(BOOTLOADER) -device loader,file=$(KERNEL_BIN_DEBUG),addr=$(KERNEL_ENTRY_PA) -s -S" && \
		tmux split-window -h -p 90 "$(GDB) -ex 'file $(KERNEL_ELF_DEBUG)' -ex 'set arch riscv:rv64' -ex 'target remote localhost:1234' -ex 'source ./scripts/debug.gdb'" && \
		tmux -2 attach-session -d

gdb: build
	$(GDB) \
		-ex 'file $(KERNEL_ELF_DEBUG)' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote localhost:1234'\
		-ex 'source ./scripts/debug.gdb'
	
clean:
	@cd ../user && make clean
	@cargo clean		
	@rm *.log


build-tests:
	@echo "build tests"
	@cd ../user && make build
	@echo Platform: $(BOARD)
	@sed 's/#BASE_ADDRESS/$(KERNEL_ENTRY_PA)/' src/$(LINKER_SCRIPT_TEMPLATE) > src/$(LINKER_SCRIPT)
	@LOG=$(LOG) cargo build --tests --features test
	@rm src/$(LINKER_SCRIPT)
	@echo $(KERNEL_TEST_ELF)
	@echo $(KERNEL_TEST_BIN)
	$(OBJCOPY) $(KERNEL_TEST_ELF) --strip-all -O binary $(KERNEL_TEST_BIN)

# Run tests in QEMU
test: build-tests
	@echo "Running tests in QEMU..."
	$(QEMU) \
		-machine virt \
		-nographic \
		-bios $(BOOTLOADER) \
		-device loader,file=$(KERNEL_TEST_BIN),addr=$(KERNEL_ENTRY_PA) || rm $(KERNEL_TEST_BIN)
	
	
disasm: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) | less

disasm-vim: kernel
	@$(OBJDUMP) $(DISASM) $(KERNEL_ELF) > $(DISASM_TMP)
	@nvim $(DISASM_TMP)
	@rm $(DISASM_TMP)

packfs:
	cd ../easy-fs-fuse && cargo run -- -s ../user/src/bin -t ../user/target/riscv64gc-unknown-none-elf/release/
