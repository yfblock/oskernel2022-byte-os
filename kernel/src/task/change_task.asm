# 我们将会用一个宏来用循环保存寄存器。这是必要的设置
.altmacro
# 寄存器宽度对应的字节数
.set    REG_SIZE, 8
# Context 的大小
.set    CONTEXT_SIZE, 34

# 宏：将寄存器存到栈上
.macro SAVE reg, offset
    sd  \reg, \offset*8(sp)
.endm

.macro SAVE_N n
    SAVE  x\n, \n
.endm

.macro SAVE_GP reg, offset
    sd  \reg, \offset*8(gp)
.endm

.macro SAVE_GP_N n
    SAVE_GP  x\n, \n
.endm


# 宏：将寄存器从栈中取出
.macro LOAD reg, offset
    ld  \reg, \offset*8(sp)
.endm

.macro LOAD_N n
    LOAD  x\n, \n
.endm

    .section .text
    .global change_task
change_task:
    # 申请栈空间
    addi sp, sp, -32*8
    
    csrrw a0, satp, a0
    # 保存x1寄存器
    SAVE_N 1
    # 保存x3寄存器
    SAVE_N 3
    # 保存x5-想1寄存器
    .set n, 4
    .rept 27
        SAVE_N %n
        .set n, n+1
    .endr

    la a0, __task_restore
    csrw stvec, a0

    # la a0, int_callback_entry
    # csrw stvec, a0

    csrw sscratch, sp
    mv sp, a1

    # 恢复 CSR
    LOAD    t0, 32
    LOAD    t1, 33
    # LOAD    t2, 2

    csrw sstatus, t0
    csrw sepc, t1
    # csrw sscratch, t2

    # 恢复通用寄存器
    LOAD    x1, 1

    # 恢复 x3 至 x31
    .set    n, 3
    .rept   29
        LOAD_N  %n
        .set    n, n + 1
    .endr

    # 恢复 sp（又名 x2）这里最后恢复是为了上面可以正常使用 LOAD 宏
    LOAD    x2, 2
    # csrrw sp, sscratch, sp
    sfence.vma
    sret

.global __task_restore
__task_restore:
    csrrw sp, sscratch, sp

    # 因为sp 0 和 2未使用所以 存在这里无事
    sd gp, 0(sp)
    ld gp, 11*8(sp) # 加载从x11保存的 context地址

__store_task_context:
    # 保存x1寄存器
    SAVE_GP_N 1
    # 保存x3寄存器
    SAVE_GP_N 3
    # 保存x5-想1寄存器
    .set n, 5
    .rept 27
        SAVE_GP_N %n
        .set n, n+1
    .endr
    # 保存寄存器信息
    csrr t0, sstatus
    csrr t1, sepc
    csrr t2, sscratch
    sd t0, 32*8(gp)
    sd t1, 33*8(gp)
    # 读取用户栈信息 写入context
    sd t2, 2*8(gp)

    # 将gp从sp中load
    ld a0, 0(sp)
    sd a0, 4*8(gp)

__load_kernel_context:
    # 恢复信息
    LOAD_N 1
    LOAD_N 3
    .set n, 4
    .rept 27
        LOAD_N %n
        .set n, n+1
    .endr

    csrrw a0, satp, a0
    sfence.vma

    la a0, int_callback_entry
    csrw stvec, a0
    
    # 回收栈
    addi sp, sp, 32*8
    ret