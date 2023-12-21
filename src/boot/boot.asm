global start
extern _start

section .text
start:              ; this is where the bootloader jumps to in order to start the kernel
    push ebx        ; save ebx (contains the multiboot info structure) so we can use it later
    push eax        ; save eax (contains the magic number) so we can use it later
    call _start     ; call the kernel's entry point
    cli             ; disable interrupts
halt:               ; halt the CPU by going into an infinite loop (waiting for an interrupt)
    hlt
    jmp halt