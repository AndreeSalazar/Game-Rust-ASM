; fixed_point.asm - Fixed-point math operations
; Author: Eddi Andreé Salazar Matos
; 16.16 fixed-point format
;
; Useful for deterministic physics calculations

default rel
bits 64

section .text

; Fixed-point multiply (16.16)
; RCX = a, RDX = b
; Returns: RAX = (a * b) >> 16
global fixed_mul
fixed_mul:
    mov     rax, rcx
    imul    rdx             ; RDX:RAX = a * b (64-bit result)
    shrd    rax, rdx, 16    ; Shift right by 16 bits
    ret

; Fixed-point divide (16.16)
; RCX = a, RDX = b
; Returns: RAX = (a << 16) / b
global fixed_div
fixed_div:
    test    rdx, rdx
    jz      .div_zero
    
    mov     rax, rcx
    shl     rax, 16         ; a << 16
    cqo                     ; Sign extend RAX into RDX:RAX
    idiv    rdx             ; RAX = (a << 16) / b
    ret
    
.div_zero:
    xor     rax, rax
    ret

; Fixed-point square root (16.16)
; RCX = value
; Returns: RAX = sqrt(value) in 16.16
global fixed_sqrt
fixed_sqrt:
    test    rcx, rcx
    jle     .zero
    
    ; Convert to float, sqrt, convert back
    cvtsi2ss xmm0, rcx      ; Convert to float
    mov     eax, 65536
    cvtsi2ss xmm1, eax
    divss   xmm0, xmm1      ; Divide by 65536 to get actual value
    sqrtss  xmm0, xmm0      ; Square root
    mulss   xmm0, xmm1      ; Multiply by 65536 for fixed-point
    cvttss2si rax, xmm0     ; Convert back to int
    ret
    
.zero:
    xor     rax, rax
    ret

; Batch fixed-point multiply
; RCX = a (i32*), RDX = b (i32*), R8 = out (i32*), R9 = count
global fixed_mul_batch
fixed_mul_batch:
    test    r9, r9
    jz      .done
    
.loop:
    movsxd  rax, dword [rcx]
    movsxd  r10, dword [rdx]
    imul    r10
    shrd    rax, rdx, 16
    mov     [r8], eax
    
    add     rcx, 4
    add     rdx, 4
    add     r8, 4
    dec     r9
    jnz     .loop
    
.done:
    ret

; Linear interpolation (fixed-point)
; RCX = a, RDX = b, R8 = t (0-65536 = 0.0-1.0)
; Returns: RAX = a + (b - a) * t
global fixed_lerp
fixed_lerp:
    mov     rax, rdx
    sub     rax, rcx        ; b - a
    imul    r8              ; (b - a) * t
    shrd    rax, rdx, 16    ; >> 16
    add     rax, rcx        ; + a
    ret
