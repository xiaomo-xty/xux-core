#![no_std]
#![no_main]

use user::{println, test_syscall};
const PAGE_SIZE: usize = 4096;
const TARGET_SIZE: usize = PAGE_SIZE * 2 + 512; // 8.5KB 确保跨页
const PATTERN: &str = "CROSS-PAGE-TEST|"; // 16字节模式

// mut adjuct user stack size to greater than 2*PAGESIZE


#[no_mangle]
fn main() -> i32{
    println!("test syscall");

    let pattern_len = PATTERN.len();
    let repeat_times = TARGET_SIZE / pattern_len + 1; // 向上取整
    
    // 2. 构造缓冲区
    let mut buffer = [0u8; TARGET_SIZE];
    
    // 3. 填充模式（高效循环展开）
    for i in 0..repeat_times {
        let start = i * pattern_len;
        let end = (i + 1) * pattern_len;
        
        if end <= TARGET_SIZE {
            buffer[start..end].copy_from_slice(PATTERN.as_bytes());
        } else {
            // 处理最后不完整的块
            let remaining = TARGET_SIZE - start;
            buffer[start..].copy_from_slice(&PATTERN.as_bytes()[..remaining]);
        }
    }
    test_syscall(&buffer);
    0
}