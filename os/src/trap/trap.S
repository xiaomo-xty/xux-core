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
    addi sp, sp, -34*8

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


    # ======================STEP [3]=================================
    # |                 Calling trap_handler                        |
    # |_____________________________________________________________|
    # (cx: &mut TrapContext)
    mv a0, sp
    call trap_handler



# fn __restore(ctx_addr: usize);
#   - case1: start running app by __restore
#   - case2: back to U after handling trap 
#   ctx_addr: usize
# __restore(KERNEL_STACK.push_context(...))
__restore:
    # mv sp, a0
    # now sp -> kernel stack(after allocated)
    # sscratch -> user stack

    # Restore the saved context after handling the trap
    ld t0, 32*8(sp)   # Restore sstatus
    ld t1, 33*8(sp)   # Restore spec
    ld t2, 2*8(sp)    # Restore sscratch

    csrw sstatus, t0  # Restore sstatus
    csrw sepc, t1     # Restore spec to sepc (return address)
    csrw sscratch, t2 # Restore sscratch (user stack pointer)

    # Restore the general-purpose registers
    # except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)

    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    # Release the `TrapContext` from the stack
    addi sp, sp, 34*8

    # Switch back to the user stack
    csrrw sp, sscratch, sp
    sret