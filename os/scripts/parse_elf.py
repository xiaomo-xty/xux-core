from elftools.elf.elffile import ELFFile

def parse_elf(file_path):
    with open(file_path, 'rb') as f:
        elf = ELFFile(f)
        
        segments = []
        for section in elf.iter_sections():
            name = section.name
            addr = section['sh_addr']
            size = section['sh_size']
            segments.append((name, addr, size))
        
        return segments

# 使用方法
elf_file = "target/riscv64gc-unknown-none-elf/release/os"
segments = parse_elf(elf_file)
for name, addr, size in segments:
    print(f"Segment: {name}, Address: {hex(addr)}, Size: {size}")
