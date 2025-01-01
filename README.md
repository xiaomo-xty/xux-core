# Project Overview
This is a RISC-V based operating system project that includes a kernel built with Rust. 
The kernel is designed to run on either a QEMU emulator or a K210 hardware board, 
depending on the selected configuration. The project includes the bootloader, 
kernel, and necessary tooling for debugging and flashing the system to the target platform.

# Building
The `Makefile` defines various build targets and configurations. To build the kernel and 
generate the binary, use the following make command:

```bash
make build
```

## Key Variables
- `OS`: The name of the operating system defined in `Cargo.toml`.
- `TARGET`: The target architecture (e.g., `riscv64gc-unknown-none-elf`).
- `MODE`: The build mode, either `release` or `debug`.
- `LOG`: The log level for the kernel, default is `INFO`.
- `BOARD`: The target platform, either `qemu` or `k210`.
- `SBI`: The name of the RustSBI bootloader used in the project.


## Build Steps
1. The build process first configures the `linker.ld` file based on the platform's 
   base address.
2. The kernel is then compiled using `cargo build`, with either `--release` or `--debug` 
   flags based on the `MODE` variable.
3. The compiled ELF file is converted to a binary format using `rust-objcopy`.
4. The resulting binary is stored in `$(KERNEL_BIN)`.


## Running the Kernel
To run the kernel, you can use either QEMU for emulation or the K210 board for actual hardware.

### Running on QEMU
To run the kernel in the QEMU emulator, use the following command:

```bash
make run BOARD=qemu
```

### Running on K210
To run the kernel on the K210 hardware, you must first flash the kernel binary to the board. 
Use the following commands:

```bash
make run BOARD=k210
```

This will flash the bootloader and kernel binary to the K210 board and start the kernel.

## Debugging
To debug the kernel, use the following command to start QEMU with debugging enabled:

```bash
make debug
```

This will launch QEMU and start a GDB server, which you can connect to using GDB.
You can use the following command to start GDB with the correct configuration:

```bash
make gdb
```

## Disassembly
To generate a disassembly of the kernel, use the following command:

```bash
make disasm
```

This will generate the disassembly of the kernel and display it using `less`.

## Clean
To clean the project and remove all generated files, use the following command:

```bash
make clean
```

## Testing
To display the current board configuration and bootloader size, use:

```bash
make test
```

## Tools
- **QEMU**: A generic and open-source machine emulator and virtualizer.
- **rust-objdump**: A tool for disassembling Rust binaries.
- **rust-objcopy**: A tool for copying and converting Rust binaries.
- **KFlash**: A Python script used for flashing the kernel to the K210 hardware.
- **GDB**: The GNU Debugger used for debugging the kernel.

## File Structure
- `src/`: Contains the kernel source code.
- `target/`: The build output directory, containing the compiled kernel and binary.
- `tools/`: Contains tools like `qemu` and `kflash.py`.
- `bootloader/`: Contains the bootloader used for the K210 board.
- `Cargo.toml`: The project configuration file, defining the dependencies and build settings.

