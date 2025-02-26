.altmacro

# sp[n] = reg.x<n> 
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm

# reg.x<n> = sp[n] 
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm


    .section .text
    .globl __alltraps
    .globl __restore
    .align 2

# `__alltraps` is set as the trap handler in the stvec register 
# during the `trap::init()` function.
__alltraps:
    # (sp)<->(sscratch) 
    # Switch to the kernel stack
    csrrw sp, sscratch, sp
    # Now, sp -> kernel stack
    # sscratch -> user stack

    // Allocated memory for `TrapContext`
    # addi sp, sp, -34*8
    // Now sp->*TrapContext in user space,
    // sscratch->user stack

# The layout of `TrapContext` in memory is as follows:
# ---------------------------------------------------------+
# |   TrapContext Address (stack top) = sp                 | <- offset 0x0 (stack top)
# +--------------------------------------------------------+
# |                   General-purpose registers            | 
# | ┌─────────────────────────────────────────────────────┐| <- offset 0x0
# | │   x[0] (usize)              SKIP                    │|
# | ├─────────────────────────────────────────────────────┤|
# | │   x[1] (usize)                                      │|
# | ├─────────────────────────────────────────────────────┤|
# | │   x[2] (usize)              SKIP                    │|
# | ├─────────────────────────────────────────────────────┤|
# | │       ...                                           │|
# | ├─────────────────────────────────────────────────────┤|
# | │   x[30] (usize)                                     │|
# | ├─────────────────────────────────────────────────────┤|
# | │   x[31] (usize)                                     │|
# | └─────────────────────────────────────────────────────┘|
# +--------------------------------------------------------+ <- offset 0x100
# |                       sstatus (Sstatus)                |    
# | ┌─────────────────────────────────────────────────────┐|
# | │   sstatus (usize)          (user stack pointer)     │|
# | └─────────────────────────────────────────────────────┘|
# |    (sstatus stores the previous stack pointer sp,      |
# |     saved in sscratch during trap entry)               |
# +--------------------------------------------------------+ <- offset 0x108
# |                       spec (usize)                     |    
# | ┌─────────────────────────────────────────────────────┐|
# | │   spec (usize)             (return address)         │|
# | └─────────────────────────────────────────────────────┘|
# |   (spec stores the return address for resuming program,|
# |   it is restored to sepc in `__restore` for resuming  |
# |   the program after trap handling)                     |
# +--------------------------------------------------------+

    # ======================STEP [1]=================================
    # |        Save general registers , except tp, sp, and x0       |
    # |_____________________________________________________________|
    # Skip x0 = 0 (x0 is always zero and doesn't need to be saved)
    # Save x1 to x31 (general-purpose registers) to the kernel stack
    sd x1, 1*8(sp)
    # Skip x2 (sp), as it is saved in sscratch
    sd x3, 3*8(sp)
    # Skip x3 (tp), as it is unnessasary
    # Save general-purpose registers x5 to x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # ======================STEP [2]=================================
    # |                Save special registers                       |
    # |_____________________________________________________________|
    # Save the supervisor status register (sstatus) to the stack
    csrr t0, sstatus
    # Save the supervisor exception program counter (sepc) to the stack
    csrr t1, sepc

     # Save sstatus, sepc, and sscratch to the kernel stack
    sd t0, 32*8(sp) # Save sstatus
    sd t1, 33*8(sp) # Save sepc (return address)

    # Save sscratch (user stack pointer) to the stack
    csrr t2, sscratch
    sd t2, 2*8(sp)  # Save sscratch (user stack pointer)

    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    

    # ======================STEP [3]=================================
    # |                 Calling trap_handler                        |
    # |_____________________________________________________________|
    # (cx: &mut TrapContext)
    
    # switch to kernel space
    csrw satp, t0
    sfence.vma        
    jr t1



# fn __restore(ctx_addr: usize);
#   - case1: start running app by __restore
#   - case2: back to U after handling trap 
#   ctx_addr: usize
# __restore(KERNEL_STACK.push_context(...))
__restore:
    # a0: *TrapContext in user space(Constant);
    # a1: user space token.
    # switch to user space

    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0

    # Now sp points to TrapContext in user space
    # start restoring based on it
    ld t0, 32*8(sp)   # Restore sstatus
    ld t1, 33*8(sp)   # Restore spec

    csrw sstatus, t0  # Restore sstatus
    csrw sepc, t1     # Restore spec to sepc (return address)

    # Restore the general-purpose registers
    # except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)

    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # back to user stack
    ld sp, 2*8(sp)
    sret