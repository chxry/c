;include init.asm
mov .start %sp
mov 1 %a
jmp .start
dn 0 512
.start
	add 1 %a
	add 1 %c
	mov 1 %b
.test
	add 1 %b
	mov %a %d
	div %b %d
	cmp %im 0
	jeq .start
	cmp %b %c
	jne .test
	out %a 1
	jmp .start
