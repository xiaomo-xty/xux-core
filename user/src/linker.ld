
/* config in .cargo/config.toml */

/* taget arch: riscv  */
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* BASE_ADDRESS Setting */
BASE_ADDRESS = 0x10000;

SECTIONS
{
    . = BASE_ADDRESS;

    .text : {
        /* _start would be placed on here */
        *(.text.entry)
        *(.text .text.*)
    }

    . = ALIGN(4K)
    .rodata : {
        *(.rodata .rodata.*)
        /* *(.srodata .srodata.*) */
    }


    /* 
        The .data and .bss segments 
        have the same access permissions (read/write), 
        so there is no need for page alignment between them. 
    */
    . = ALIGN(4K)
    .data : {
        *(.data .data.*)
        /* *(sdata .sdata.*) */
    }
    .bss : {
        /* start_bss = .; */
        *(.bss .bss.*)
        /* *(.sbss .sbss.*) */
        /* end_bss = .; */
    }


    /* This section is used to discard 
    specific data during the linking process. */
    /DISCARD/ : {
        *(.eh_frame)
        *(.debug*)
    }
}

/* 如果你的目标是开发一个操作系统内核（根据之前的描述，这是基于 RISC-V 的类 Unix 内核），建议保留 4K 对齐，因为：

它对内存管理、分页、性能优化都至关重要。
即使暂时没有分页需求，4K 对齐是一个良好的设计习惯，未来如果扩展分页支持，不需要重新设计链接脚本。
如果目标是更轻量级的用户态程序，则可以选择不对齐以节省空间。 */
