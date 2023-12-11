ENTRY(start)

SECTIONS {
	.idt 0x00000000 : {
		*(.idt)
	}

	.gdt 0x00000800 : {
		*(.gdt)
	}

	. = 1M;

	.boot :
	{
		*(.multiboot_header)
	}

	.text :
	{
		*(.text .text.*)
	}

	.rodata :
	{
		*(.rodata .rodata.*)
	}

	.data : 
	{
		*(.data .data.*)
	}

	.bss :
	{
		*(.bss .bss.*)
		*(COMMON)
	}
}
