global start

extern _start

section .text
bits 32

start:
	call _start
	hlt