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

; from musl e346ff86c8faee901a7c2d502b5beb983b99f972
global memset

memset:
        movzx rax, sil
        mov r8, 0x101010101010101,
        imul rax, r8

        cmp rdx, 126
        ja __L0

        test edx, edx
        jz __L1

        mov [rdi], sil
        mov [rdi + rdx - 1], sil
        cmp edx, 2
        jbe __L1

        mov [rdi + 1], ax
        mov [rdi + rdx - 1 - 2], ax
        cmp edx, 6
        jbe __L1

        mov [rdi + 1 + 2], eax
        mov [rdi + rdx - 1 - 2 - 4], eax
        cmp edx, 14
        jbe __L1

        mov [rdi + 1 + 2 + 4], rax
        mov [rdi + rdx - 1 - 2 - 4 - 8], rax
        cmp edx, 30
        jbe __L1

        mov [rdi + 1 + 2 + 4 + 8], rax
        mov [rdi + 1 + 2 + 4 + 8 + 8], rax
        mov [rdi + rdx - 1 - 2 - 4 - 8 - 16], rax
        mov [rdi + rdx - 1 - 2 - 4 - 8 - 8], rax
        cmp edx, 62
        jbe __L1

        mov [rdi + 1 + 2 + 4 + 8 + 16], rax
        mov [rdi + 1 + 2 + 4 + 8 + 16 + 8], rax
        mov [rdi + 1 + 2 + 4 + 8 + 16 + 16], rax
        mov [rdi + 1 + 2 + 4 + 8 + 16 + 24], rax
        mov [rdi + rdx - 1 - 2 - 4 -  8 - 16 - 32], rax
        mov [rdi + rdx - 1 - 2 - 4 -  8 - 16 - 24], rax
        mov [rdi + rdx - 1 - 2 - 4 -  8 - 16 - 16], rax
        mov [rdi + rdx - 1 - 2 - 4 -  8 - 16 - 8], rax

__L1:   mov rax, rdi
        ret

__L0:   test edi, 15
        mov r8, rdi
        mov [rdi + rdx - 8], rax
        mov rcx, rdx
        jnz __L2

__L3:   shr rcx, 3
        rep
        stosq
        mov rax, r8
        ret

__L2:   xor edx, edx
        sub edx, edi
        and edx, 15
        mov [rdi], rax
        mov [rdi + 8], rax
        sub rcx, rdx
        add rdi, rdx
        jmp __L3
