global start
extern _start

section .text
start:
    push ebx
    push eax
    call _start
    cli
halt:
    hlt
    jmp halt

section .bss
align 16
stack_space: resb 4096
stack_top:
