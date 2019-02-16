/* For paging related structures */

.set KERNEL_BASE, 0xFFFFFFFF80000000

.section .padata, "aw"
.align 8
.extern pd 

.globl user_pdpt
user_pdpt:
  .quad user_pd - KERNEL_BASE + 3
  .rept 511
    .quad 0
  .endr

.globl kernel_pdpt 
kernel_pdpt:
  .rept 510
    .quad 0
  .endr 
  .quad kernel_pd - KERNEL_BASE + 3
  .quad 0

.global kernel_pd
kernel_pd:
  .quad 0x0 | 0x80 | 3
  .quad 0x200000 | 0x80 | 3
  .quad 0x400000 | 0x80 | 3
  .rept 509
    .quad 0
  .endr

.global user_pd
user_pd:
  .rept 512
    .quad 0
  .endr 
