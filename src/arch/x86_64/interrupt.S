
.macro isr_stub_noerr v
  isr_entry\v:
    push $0
    push $\v
    call save_context
    jmp int_common_entry
.endm

.macro isr_stub_err v
  isr_entry\v:
    push $\v
    call save_context
    jmp int_common_entry
.endm

.macro pushaq
  push %rax
  push %rdx
  push %rcx
  push %rbx
  push %rbp
  push %rsi
  push %rdi
  push %r8
  push %r9
  push %r10
  push %r11
  push %r12
  push %r13
  push %r14
  push %r15
.endm

.macro popaq
  pop %r15
  pop %r14
  pop %r13
  pop %r12
  pop %r11
  pop %r10
  pop %r9
  pop %r8
  pop %rdi
  pop %rsi
  pop %rbp
  pop %rbx
  pop %rdx
  pop %rcx
  pop %rax
.endm

.altmacro

.section .text
.extern int_handler
.extern save_context
.align 4
/* rsp -> int vector, rsp + 8 -> error code */ 
int_common_entry:
  pushaq

  movq 15*8(%rsp), %rdi   /* int vector */
  movq 16*8(%rsp), %rsi   /* error code */
  
  mov %ds, %rax 
  push %rax
  mov %es, %rax 
  push %rax 
  push %fs
  push %gs
  mov $0x10, %rax
  mov %ax, %ds
  mov %ax, %es
  mov %ax, %fs
  mov %ax, %gs 

  call int_handler

  pop %gs
  pop %fs
  pop %rax 
  mov %ax, %es 
  pop %rax
  mov %ax, %ds

  popaq
  add $0x10, %rsp
  iretq

isr_stub_noerr 0 /* Divide error */
isr_stub_noerr 1 /* Debug exception */
isr_stub_noerr 2 /* NMI Interrupt */
isr_stub_noerr 3 /* Breakpoint */
isr_stub_noerr 4 /* Overflow */
isr_stub_noerr 5 /* BOUND range exceeded */
isr_stub_noerr 6 /* Invalid opcode */
isr_stub_noerr 7 /* Device Not Available */
isr_stub_err 8 /* Double fault */
isr_stub_noerr 9 /* Coprocessor segment overrun */
isr_stub_err 10 /* Invalid TSS */
isr_stub_err 11 /* Segment not present */
isr_stub_err 12 /* Stack segment fault */
isr_stub_err 13 /* General protection */
isr_stub_err 14 /* Page fault */
isr_stub_noerr 15 /* (reserved) */
isr_stub_noerr 16 /* x87 FPU Floating-Point error */
isr_stub_err 17 /* Alignment check */
isr_stub_noerr 18 /* Machine check */
isr_stub_noerr 19 /* SIMD Floating-Point exception */
isr_stub_noerr 20 /* Virtualization exception */
.set i, 21
.rept 256-21
  isr_stub_noerr %i
  .set i, i + 1
.endr

.globl idt_init
idt_init:
  lea idt_entry, %rax
  lea idt_handlers, %r10
  xor %r9d, %r9d

 set_loop:
  mov (%r10), %r8
  mov %r8d, %ecx
  and $0xffff, %ecx
  or $(0x8 << 16), %ecx
  movl %ecx, 0(%rax)
  mov %r8d, %ecx
  and $0xffff0000, %ecx
  or $((0x80 | 0xe) << 8), %ecx
  movl %ecx, 4(%rax)
  mov %r8, %rcx
  shr $32, %rcx
  movl %ecx, 8(%rax)
  xor %ecx, %ecx
  movl %ecx, 12(%rax)

  add $0x8, %r10
  add $0x10, %rax
  inc %r9d
  cmp $0x100, %r9d
  jnz set_loop

  lidt idt_descriptor

  ret

.section .data
.align 1

.macro handler v
  .quad isr_entry\v
.endm

.globl idt_handlers
idt_handlers:
  .set i, 0
  .rept 256
    handler %i
    .set i, i + 1
  .endr

.globl idt_entry
idt_entry:
  .rept 256
    .quad 0
    .quad 0
  .endr
idt_entry_end:

.globl idt_descriptor
idt_descriptor:
.word idt_entry_end - idt_entry - 1
.quad idt_entry
