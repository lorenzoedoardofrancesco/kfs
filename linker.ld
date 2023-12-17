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

	. = 1M;

	.multiboot_header ALIGN(8) : {
		KEEP(*(.multiboot_header))
	}

	.text ALIGN(0x1000) :
	{
		*(.text .text.*)
	}

	.rodata ALIGN(16) :
	{
		*(.rodata .rodata.*)
	}

	.data ALIGN(16) :
	{
		*(.data .data.*)
	}

	.bss ALIGN(16) :
	{
		*(.bss .bss.*)
		*(COMMON)
	}
}
