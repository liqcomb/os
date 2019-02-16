// Multiboot header
// https://www.gnu.org/software/grub/manual/multiboot/multiboot.html#Specification
.set KERNEL_BASE, 0xFFFFFFFF80000000
.set MULTIBOOT_PAGE_ALIGN, (1<<0)
.set MULTIBOOT_MEMORY_INFO, (1<<1)
.set MULTIBOOT_REQVIDMODE, (1<<2)
.set MULTIBOOT_LOADIMAGE, (1<<16)
.set MULTIBOOT_FLAGS, (MULTIBOOT_PAGE_ALIGN)
.set MULTIBOOT_MAGIC, 0x1BADB002
.set MULTIBOOT_CHECKSUM, -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)

.section .multiboot, "a"
.align 4
.globl multiboot_header
multiboot_header:
.long MULTIBOOT_MAGIC
.long MULTIBOOT_FLAGS
.long MULTIBOOT_CHECKSUM
.long multiboot_header, 0, 0, 0, 0
.long 0, 0, 0, 0

/* Entry point from multiboot */
/* Protected mode enabled, A20 line enabled */
.section .inittext, "ax"
.code32
.globl _start
_start:
  // Since --gc-sections flag removes unused sections, we need to at least hold  reference to .multiboot section
  lea multiboot_header, %edx

  // multiboot check
  cmp $0x2BADB002, %eax
  jnz failure

  // Get highest support function
  movl $0x80000000, %eax
  cpuid
  cmpl $0x80000001, %eax
  jbe failure
  
  // Get processor feature
  movl $0x80000001, %eax
  cpuid
  test $0x20000000, %edx
  jz failure
  
  // Enable PSE, PAE, PGE
  mov %cr4, %eax
  or $((1 << 4)|(1 << 5)|(1 << 7)), %eax
  mov %eax, %cr4

  // Set pml4
  lea pml4, %eax
  mov %eax, %cr3

  // Enable Long mode
  mov $0xC0000080, %ecx
  rdmsr
  or $((1 << 0)|(1 << 8)|(1 << 11)), %eax
  wrmsr

  // Enable PG, WP
  mov %cr0, %eax
  or $0x80010000, %eax
  mov %eax, %cr0

  lgdt gdt32
  ljmp $0x8, $_start64

failure:
  movw $0x100 | 'F', 0xb8000
  movw $0x100 | 'a', 0xb8002
  movw $0x100 | 'i', 0xb8004
  movw $0x100 | 'l', 0xb8006
  movw $0x100 | 'e', 0xb8008
  movw $0x100 | 'd', 0xb800a
  movw $0x100 | '.', 0xb800c
  jmp .

.code64
_start64:
  movq $_start64_high, %rax
  jmp *%rax

.section .text, "ax"
.code64
_start64_high:
  lgdt gdt64  /* loads gdt at high virtual address */

  xor %rax, %rax
  movq %rax, pml4

  mov $0x10, %ax
  mov %ax, %ss
  mov %ax, %ds
  mov %ax, %es
  mov %ax, %fs
  mov %ax, %gs

  movq $init_stack_end, %rsp

  call kentry

  hlt
  jmp .

/* initdata holds page table & gdt */
.section .initdata, "aw"
.align 8
.extern kernel_pdpt
.extern kernel_pd 

.globl pml4
pml4:
  .quad init_pdpt + 3
  .rept 510
    .quad 0
  .endr
  .quad kernel_pdpt - KERNEL_BASE + 3

init_pdpt:
  .quad kernel_pd - KERNEL_BASE + 3
  .rept 511
    .quad 0
  .endr

gdt32:
  .word gdt_end - gdt - 1
  .long gdt - KERNEL_BASE

.section .data, "aw"
.align 8
.globl gdt64
/* this will be in effect after PG enabled */
gdt64:
  .word gdt_end - gdt - 1
  .quad gdt

.globl gdt
gdt:
  .long 0, 0
  .long 0x00000000, 0x00209A00    /* 0x08: 64-bit Code */
  .long 0x00000000, 0x00009200    /* 0x10: 64-bit Data */
  .long 0x00000000, 0x0040FA00    /* 0x18: 32-bit User Code */
  .long 0x00000000, 0x0040F200    /* 0x20: User Data        */
  .long 0x00000000, 0x0020FA00    /* 0x28: 64-bit User Code       */
  .long 0x00000000, 0x0000F200    /* 0x30: User Data (64 version) */
  .rept 3
    .long 0, 0
  .endr
gdt_end:

.section .padata, "aw"
.align 8
init_stack:
  .rept 4096
    .quad 0
  .endr
.globl init_stack_end
init_stack_end:

