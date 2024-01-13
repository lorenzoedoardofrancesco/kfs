global start
extern _start
extern _kernel_start
extern _kernel_end

; Allocate the initial stack.
section .bootstrap_stack
align 16
stack_bottom:
times 16384 db 0 ; 16 KiB
stack_top:

; Preallocate pages used for paging. Don't hard-code addresses and assume they
; are available, as the bootloader might have loaded its multiboot structures or
; modules there. This lets the bootloader know it must avoid the addresses.
section .bss
align 4096
boot_page_directory:
    resb 4096
boot_page_table:
	resb 4096
; Further page tables may be required if the kernel grows beyond 3 MiB.

; The kernel entry point.
section .boot
start:
	cli
	; Physical address of boot_page_table.
	mov edi, boot_page_table - 0xC0000000
	; First address to map is address 0.
	mov esi, 0
	; Map 1023 pages. The 1024th will be the VGA text buffer.
	mov ecx, 1024

.loop_start:
    ; Only map the kernel.
    cmp esi, 0
    jl .skip_mapping
    cmp esi, _kernel_end - 0xC0000000
    jge .end_mapping

    ; Map physical address as "present, writable".
    mov edx, esi
    or edx, 0x003
    mov [edi], edx

.skip_mapping:
    ; Size of page is 4096 bytes.
    add esi, 4096
    ; Size of entries in boot_page_table is 4 bytes.
    add edi, 4
    ; Loop to the next entry if we haven't finished.
    loop .loop_start


.end_mapping:
    ; Map the page table to both virtual addresses 0x00000000 and 0xC0000000.
    mov dword [boot_page_directory - 0xC0000000], boot_page_table - 0xC0000000 + 0x003
    mov dword [boot_page_directory - 0xC0000000 + 768 * 4], boot_page_table - 0xC0000000 + 0x003

    ; Set cr3 to the address of the boot_page_directory.
    mov ecx, boot_page_directory - 0xC0000000
    mov cr3, ecx

    ; Enable paging and the write-protect bit.
    mov ecx, cr0
    or ecx, 0x80010000
    mov cr0, ecx

    ; Jump to higher half with an absolute jump.
    lea ecx, [rel .higher_half]
    jmp ecx

section .text

.higher_half:
    ; At this point, paging is fully set up and enabled.

    ; Unmap the identity mapping as it is now unnecessary.
    mov dword [boot_page_directory], 0

    ; Reload cr3 to force a TLB flush so the changes take effect.
    mov ecx, cr3
    mov cr3, ecx

    ; Set up the stack.
    mov esp, stack_top

    ; Enter the high-level kernel.
    call _start

    ; Infinite loop if the system has nothing more to do.
    cli


halt:               ; halt the CPU by going into an infinite loop (waiting for an interrupt)
    hlt
    jmp halt