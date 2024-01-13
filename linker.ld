HIGHER_HALF_OFFSET = 0xC0000000;

ENTRY(start)

SECTIONS {
    /* Place the Multiboot header at the very start of the kernel */
    .boot : {
        KEEP(*(.multiboot_header))
    }

    /* Define the start of the kernel in the higher half */
    . = HIGHER_HALF_OFFSET + 1M;
    _kernel_start = .;

    /* Place the text section after the kernel start */
    .text ALIGN(4K) : AT(ADDR(.text) - HIGHER_HALF_OFFSET) {
        *(.text .text.*)
    }

    /* Place other sections */
    .rodata ALIGN(4K) : AT(ADDR(.rodata) - HIGHER_HALF_OFFSET) {
        *(.rodata .rodata.*)
    }

    .data ALIGN(4K) : AT(ADDR(.data) - HIGHER_HALF_OFFSET) {
        *(.data .data.*)
    }

    .bss ALIGN(4K) : AT(ADDR(.bss) - HIGHER_HALF_OFFSET) {
        *(.bss .bss.*)
        *(COMMON)
		*(.bootstrap_stack)
    }
    _kernel_end = .;
}
