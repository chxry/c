mov .start %sp
jmp .start
dn 0 512
.start
        out 1
        push 2
        call .fn
        out 3
        hlt
.fn
; clean this up
        pop %a
        pop %b
        out %b
        push %a
        ret