.section .text.entry
.globl _start

.globl boot_stack_top
.globl boot_stack_lower_bound



_start:
    la sp, boot_stack_top
    call rust_main

    # Allocate space for stack

    # * Low Address 0x00...0
    # _______________________________
    # |                               \
    # |                               |
    # |                               |
    # |      ______________________   |
    # @=====[boot_stack_lower_bound]==@            A
    # @                               @           /|\
    # @           4096*16             @----------(top)
    # @                               @            T
    # @      ______________________   @          __|___              
    # @=====[____boot_stack_top____]==@---------(bottom)
    # |                               |
    # |                               |
    # |                               |
    # v______________________________/ 
    #  High Addrees

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top: