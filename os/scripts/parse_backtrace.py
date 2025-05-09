#!/usr/bin/env python3
"""
内核地址回溯解析工具
用法: 
  1. 直接解析日志文件: ./backtrace.py <内核镜像路径> < panic.log
  2. 实时解析输出:      make run | ./backtrace.py target/os
"""

import sys
import re
import subprocess
from argparse import ArgumentParser

def parse_address(image_path, address):
    """调用 addr2line 解析地址"""
    try:
        result = subprocess.check_output(
            ['riscv64-unknown-elf-addr2line', 
             '-e', image_path,
             '-f', '-p', address],
            stderr=subprocess.DEVNULL
        ).decode().strip()
        return result
    except subprocess.CalledProcessError:
        return f"[解析失败] {address}"
    except FileNotFoundError:
        sys.exit("错误：未找到 riscv64-unknown-elf-addr2line，请检查工具链安装")

def highlight(line):
    """终端颜色高亮 (可选)"""
    return f"\033[33m{line}\033[0m"  # 黄色

def main():
    parser = ArgumentParser(description='内核地址回溯解析工具')
    parser.add_argument('--color', action='store_true', help='启用颜色高亮')
    args = parser.parse_args()

    kernel_path = "/home/littlesun/Workshop/xux-Core/os/target/riscv64gc-unknown-none-elf/debug/os"

    addr_pattern = re.compile(r'ra=(0x[0-9a-f]+)')

    for line in sys.stdin:
        line = line.rstrip()
        print(line)
        
        # 匹配回溯地址
        match = addr_pattern.search(line)
        if not match:
            continue
        
        addr = match.group(1)
        result = parse_address(kernel_path, addr)
        
        # 格式化输出
        output = f"    ↳ {result}"
        if args.color:
            output = highlight(output)
        print(output)

if __name__ == '__main__':
    main()