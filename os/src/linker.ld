
/* config in .cargo/config.toml */

/* taget arch: riscv  */
OUTPUT_ARCH(riscv)
ENTRY(_start)

/* BASE_ADDRESS Setting */
BASE_ADDRESS = 0x80200000;

SECTIONS
{
    . = BASE_ADDRESS;
    /* kernel starting from here */
    skernel = .;

    stext = .;
    .text : {
        *(.text.entry)
        *(.text .text.*)
    }

    /* End of text, begin of rodata */
    . = ALIGN(4K);
    etext = .;
    srodata = .;
    .rodata : {
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
    edata = .;
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