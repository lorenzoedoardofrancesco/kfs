; boot.asm

extern _start   ; Declare the external Rust function

global start

section .text
bits 32
start:
	mov word [0xb8016], 0x4f4b ; 'K' with orange background
	mov word [0xb8018], 0x4f46 ; 'F' with orange background
	mov word [0xb801a], 0x4f43 ; 'C' with orange background
	call _start ; Call the Rust function
	hlt