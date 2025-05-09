
/* config in .cargo/config.toml */

/* taget arch: riscv  */
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* BASE_ADDRESS Setting */
BASE_ADDRESS = #BASE_ADDRESS;

SECTIONS
{
    . = BASE_ADDRESS;
    /* kernel starting from here */
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)

        /*定义一个strampoline符号
        将其对齐到4K边界，
        预计存放 .text.trampoline 所标识的跳板代码*/
        . = ALIGN(4K);
        strampoline = .;
        *(.text.trampoline);
        . = ALIGN(4K);
        *(.text .text.*)
    }

    /* End of text, begin of rodata */
    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {

        /* 系统调用注册表（只读） */
        __syscall_registry_start = .;
        KEEP(*(.syscall_registry))
        __syscall_registry_end = .;

        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
    }

    /* End of rodata, start of data */
    . = ALIGN(4K);
    erodata = .;
    sdata = .;
    .data : {
        *(.data .data.*)
        *(sdata .sdata.*)
    }

    /* End of data*/
    . = ALIGN(4K);
    edata = .;

    .syscall_table : {
        __syscall_table_start = .;
        KEEP(*(.syscall_table))       /* 强制保留该段 */
        __syscall_table_end = .;
    }                            /* 明确读写属性 */



    . = ALIGN(4K);
    sbss_with_stack = .;
    .bss : {
        /* The .bss.stack section is outside 
         * the memory range defined by .sbss and .ebss */
        *(.bss.stack)

        /*start of bss */
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
    }


    /* End of ebss, is also end of kernel*/
    . = ALIGN(4K);
    ebss = .;
    ekernel = .;

    /* This section is used to discard 
    specific data during the linking process. */
    /DISCARD/ : {
        *(.eh_frame)
    }
}