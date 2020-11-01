; Copyright © 2005-2020 Rich Felker, et al.
;
; Permission is hereby granted, free of charge, to any person obtaining
; a copy of this software and associated documentation files (the
; "Software"), to deal in the Software without restriction, including
; without limitation the rights to use, copy, modify, merge, publish,
; distribute, sublicense, and/or sell copies of the Software, and to
; permit persons to whom the Software is furnished to do so, subject to
; the following conditions:
;
; The above copyright notice and this permission notice shall be
; included in all copies or substantial portions of the Software.
;
; THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
; EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
; MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
; IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
; CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
; TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
; SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

; from musl b35c4c475bea3c8f938d8e9696d1138eabb54a89
global memcpy

memcpy:
        mov rax, rdi
        cmp rdx, 8
        jc __L0
        test edi, 7
        jz __L0
__L1:   movsb
        dec rdx
        test edi, 7
        jnz __L1
__L0:   mov rcx, rdx
        shr rcx, 3
        rep
        movsq
        and edx, 7
        jz __L2
__L3:   movsb
        dec edx
        jnz __L3
__L2:   ret
