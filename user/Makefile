TARGET := riscv64gc-unknown-none-elf
MODE := release
APP_SRC_DIR := src/bin
TARGET_DIR := target/$(TARGET)/$(MODE)
APPS := $(wildcard $(APP_SRC_DIR)/*.rs)
ELFS := $(patsubst $(APP_SRC_DIR)/%.rs, $(TARGET_DIR)/%, $(APPS))
BINS := $(patsubst $(APP_SRC_DIR)/%.rs, $(TARGET_DIR)/%.bin, $(APPS))


OBJDUMP := rust-objdump --arch-name=riscv64
OBJCOPY := rust-objcopy --binary-architecture=riscv64

# elf: $(APPS)
# ifeq ($(MODE),debug)
# 	@cargo build
# else
# 	@cargo build --$(MODE)
# endif

elf: $(APPS)
	@cargo build --release

binary: elf
	@$(foreach elf, $(ELFS), $(OBJCOPY) $(elf) --strip-all -O binary $(patsubst $(TARGET_DIR)/%, $(TARGET_DIR)/%.bin, $(elf));)

	

build: binary

.PHONY: test clean
test:
	echo $(APPS)
	echo $(ELFS)

clean:
	@cargo clean