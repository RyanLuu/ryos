.option norvc  # disable generation of compressed instructions
.section .text.init
.global _start
.global kinit
.global kernel_vec
_start:


  # Initialize global pointer
  # https://sourceware.org/binutils/docs-2.31/as/RISC_002dV_002dDirectives.html
  .option push
  .option norelax
  la gp, __global_pointer$
  .option pop

	# Initialize stack pointer; 64K stack for each hart
	la sp, __stack_start
	li a0, 0x10000
	csrr a1, mhartid
	addi a1, a1, 1
	mul a0, a0, a1
	add sp, sp, a0

  # busy wait on all harts except 0
  csrr t0, mhartid
  bnez t0, 3f

  # zero out bss if necessary
	la 		a0, __bss_start
	la		a1, __bss_end
	bgeu	a0, a1, 2f
1:
	sd		zero, (a0)
	addi	a0, a0, 8
	bltu	a0, a1, 1b
2:

  call kinit  

3:
  wfi
  j 3b
