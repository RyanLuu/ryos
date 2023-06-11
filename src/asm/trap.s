.altmacro
.set NUM_GP_REGS, 32
.set REG_SIZE, 8
.macro save_reg i, basereg=t6
  sd x\i, ((\i)*REG_SIZE)(sp)
.endm
.macro load_reg i, basereg=t6
  ld x\i, ((\i)*REG_SIZE)(sp)
.endm

.global kernel_vec
.global kernel_trap
.align 4
kernel_vec:

  # push all general purpose registers to the stack
  addi sp, sp, -256
.set i, 0
.rept 32
  save_reg %i # write x0-x31
.set i, i+1
.endr

  # handle trap in trap.rs
  call kernel_trap

  # pop all general purpose registers from the stack
.set i, 0
.rept 32
  load_reg %i # read x0-x31
.set i, i+1
.endr
  addi sp, sp, 256

  sret
