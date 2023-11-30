mov 1 %a
.reset
	add 1 %a
	mov %a %c
	sub 1 %c
	mov 1 %b
.test
	add 1 %b
	mov %a %d
	div %b %d
; allow 2nd op of cmp to be const
	cmp 0 %im
	jeq .reset
	cmp %b %c
	jne .test
	out %a
	jmp .reset
