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