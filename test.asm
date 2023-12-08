mov .start %sp
jmp .start
dn 0 512
.start
        out 1 1
        push 2
        call .fn
        out 3 1
        hlt
.fn
; clean this up
        pop %a
        pop %b
        out %b 1
        push %a
        ret