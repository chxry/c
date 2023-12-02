mov 1 %a
.reset
	add 1 %a
	add 1 %c
	mov 1 %b
.test
	add 1 %b
	mov %a %d
	div %b %d
	cmp %im 0
	jeq .reset
	cmp %b %c
	jne .test
	out %a
	jmp .reset
