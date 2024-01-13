ENTRY (_start)

SECTIONS
{
	.boot : {
		*(.boot)
		*(.multiboot_header)
	}

	. += 0x00000000 + 1M;
	_kernel_start = .;

	.text ALIGN (4K) : AT (ADDR (.text) - 0x00000000)
	{
		*(.text)
	}
	.rodata ALIGN (4K) : AT (ADDR (.rodata) - 0x00000000)
	{
		*(.rodata)
	}
	.data ALIGN (4K) : AT (ADDR (.data) - 0x00000000)
	{
		*(.data)
	}
	.bss ALIGN (4K) : AT (ADDR (.bss) - 0x00000000)
	{
		*(COMMON)
		*(.bss)
		*(.bootstrap_stack)
	}
	/* Add a symbol that indicates the end address of the kernel. */
	_kernel_end = .;
}