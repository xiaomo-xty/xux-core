.altmacro

# (n+2)*8(a0): ctx.s[n+1]
# SAVE_SN n  : ctx.s[n+1] = reg.s<n>
.macro SAVE_SN n
    sd s\n, (\n+2)*8(a0)
.endm

# (n+2)*8(a1): ctx.s[n+1]
# LOAD_SN n : reg.s<n> = ctx.s[n+1]
.macro LOAD_SN n
    ld s\n, (\n+2)*8(a1)
.endm

    .section .text
    .globl __switch

    # TaskContext Layout in Memory:
    #
    # ┌───────────────────────────────────────┐
    # │       return address                  │ <- offset 0  (ra)  
    # │ (e.g., __restore in __switch)         │
    # ├───────────────────────────────────────┤
    # │       stack pointer                   │ <- offset 8  (sp)
    # │ (kernel stack pointer of app)         │
    # ├───────────────────────────────────────┤ <- offset 16 (s[0..11]) 
    # │ ┌─────────────────────┐               │        (callee saved registers: s0..s11)
    # │ │   saved register s0 │ <- offset 16  │ 
    # │ ├─────────────────────┤               │
    # │ │   saved register s1 │ <- offset 24  │ 
    # │ ├─────────────────────┤               │
    # │ │         ...         │               │
    # │ ├─────────────────────┤               │
    # │ │  saved register s11 │ <- offset 104 │
    # │ └─────────────────────┘               │
    # └───────────────────────────────────────┘


    # __switch
    #(
    #     current_task_cx_ptr: *mut   TaskContext,
    #            |
    #            └──>[a0]        
    #
    #     next_task_cx_ptr   : *const TaskContext
    #            |
    #            └──>[a1]     
    # )
__switch:
    # ======================STEP [1]=================================
    # |         Save [ra] and [s0~s11] to current_task_cx_ptr       |
    # |_____________________________________________________________|

    #     current_task_cx_ptr.sp = reg.sp
    sd sp, 8(a0)
    #     current_task_cx_ptr.ra = reg.ra
    sd ra, 0(a0)

    #              reg.s<0..=11>
    #                   |
    #                   v
    # next_task_cx_ptr.s[1..=12]
    .set n, 0
    .rept 12    
        SAVE_SN %n
        .set n, n+1
    .endr

    # ======================STEP [2]=================================
    # |         Restore [ra] and [s0~s11] from next_task_cx_ptr     |
    # |_____________________________________________________________|

    # ra = next_task_cx_ptr.ra
    # in generally, ra point to <trap.S __restore>, so that resotre
    ld ra, 0(a1)

    # next_task_cx_ptr.s[1..=12]
    #                   |
    #                   v
    #              reg.s<0..=11>
    .set n, 0
    .rept 12
        LOAD_SN %n
        .set n, n + 1
    .endr

    #  sp = next_task_cx_ptr.sp
    ld sp, 8(a1)
    ret

