HIGHER_HALF_OFFSET = 0xC0000000;

ENTRY(start)

SECTIONS {
	.idt 0x00000000 : {
		*(.idt)
		. = ALIGN(0x800);
	}

	.gdt 0x00000800 : {
		*(.gdt)
		. = ALIGN(4);
	}

    /* Place the Multiboot header at the very start of the kernel */
    .boot 0x1000 : {
		_multi_boot_header = .;
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
	. = ALIGN(4K);
    _kernel_end = .;
}
