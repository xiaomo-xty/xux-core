.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)

.endm
.macro LOAD_GP n
    #sp[n*8] -> xn
    ld x\n, \n*8(sp)
.endm


    .section .text
    .globl __alltraps
    .globl __restore
    .align 2
__alltraps:
    # sp<->csr
    csrrw sp, sscratch, sp
    # Now, sp -> kernel stack
    # sscratch -> user stack
    addi sp, sp, -34*8

    # Skip x0 = 0
    sd x1, 1*8(sp)
    # Skip x2, namely sp
    # it's original value in sscratch
    sd x3, 3*8(sp)

    # xn -> sp[n*8]
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    # t0 <- sstatus
    csrr t0, sstatus
    # t1 <- sepc, pc on ecll
    csrr t1, sepc
    csrr t2, sscratch

    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    sd t2, 2*8(sp)

    # set input argument of trap_handler
    # (cx: &mut TrapContext)
    mv a0, sp
    call trap_handler



# fn __restore(ctx_addr: usize);
__restore:
    # case1: start running app by __restore
    # case2: back to U after handling trap
    # ctx_addr: usize
    # __restore(KERNEL_STACK.push_context(...))
    mv sp, a0
    # now sp -> kernel stack(after allocated)
    # sscratch -> user stack

    #restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    ld t2, 2*8(sp)

    csrw sstatus, t0
    csrw sepc, t1
    csrw sscratch, t2

    # restore general-purpuse registers
    # except sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)

    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr

    #release TrapContext on kernel stack
    addi sp, sp, 34*8

    # sp <-> sscratch
    csrrw sp, sscratch, sp
    sret