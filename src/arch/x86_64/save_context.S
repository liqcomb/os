.section .text 
.extern store_context
.globl save_context
save_context:
  push %r15
  push %r14
  push %r13
  push %r12
  push %r11
  push %r10
  push %r9
  push %r8
  push %rdi
  push %rsi
  push %rbx
  push %rdx
  push %rcx
  push %rax

  mov %ds, %rax
  push %rax
  mov %es, %rax
  push %rax
  mov %fs, %rax
  push %rax 
  mov %gs, %rax
  push %rax

  push %rbp
  mov %cr3, %rax
  push %rax
  mov %rsp, %rdi

  /* Don't forget to update segment register */
  mov $0x10, %rax
  mov %ax, %ds
  mov %ax, %es
  mov %ax, %fs
  mov %ax, %gs

  call store_context

  pop %rax
  pop %rbp
  
  pop %rax
  mov %ax, %gs
  pop %rax
  mov %ax, %fs
  pop %rax
  mov %ax, %es
  pop %rax
  mov %ax, %ds

  pop %rax
  pop %rcx
  pop %rdx
  pop %rbx
  pop %rsi
  pop %rdi
  pop %r8
  pop %r9
  pop %r10
  pop %r11
  pop %r12
  pop %r13
  pop %r14
  pop %r15

  /* add $(0x8 * (6 + 14)), %rsp */
  ret
