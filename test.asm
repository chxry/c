.test 
	add 3 %a
	add %a %b
	mov %b %c
	cmp 100 %b
	jlt .test
	mov *.data %g
	hlt

.data
	dw 0x40
