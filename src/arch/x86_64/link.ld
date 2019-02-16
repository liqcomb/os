ENTRY(_start);
OUTPUT_FORMAT(elf64-x86-64);

KERNEL_BASE = 0xFFFFFFFF80000000;

SECTIONS 
{
  . = 1M;

  /* Multiboot header */
  .multiboot BLOCK(4K) : ALIGN(4K) {
    *(.multiboot)
  }

  /* Early stage text segment */
  .stext BLOCK(4K) : ALIGN(4K) {
    *(.inittext)
  }

  /* Early stage data segment */
  .sdata BLOCK(4K) : ALIGN(4K) {
    *(.initdata)
  }
  
  . += KERNEL_BASE;

  /DISCARD/ : {
    *(.note .note.*)
  }

  .text BLOCK(4K) : AT(ADDR(.text) - KERNEL_BASE) ALIGN(4K) {
    *(.text .text.*)
  }

  .rodata BLOCK(4K) : AT(ADDR(.rodata) - KERNEL_BASE) ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  .padata ALIGN(4K) : AT(ADDR(.padata) - KERNEL_BASE) ALIGN(4K) {
    *(.padata)
  }

  .data ALIGN(4K) : AT(ADDR(.data) - KERNEL_BASE) ALIGN(4K) {
    *(.data .data.*)
  }

  .bss ALIGN(4K) : AT(ADDR(.bss) - KERNEL_BASE) ALIGN(4K) {
    *(.bss .bss.*)
  }
}
