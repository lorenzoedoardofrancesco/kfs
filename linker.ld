ENTRY(start)

SECTIONS {
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
