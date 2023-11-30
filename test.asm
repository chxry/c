.test 
	add 3 %a
	add %a %b
	mov %b %c
	cmp 100 %b
	jlt .test
	mov 0x40 %g
	hlt
