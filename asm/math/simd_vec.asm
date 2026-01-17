; simd_vec.asm - SIMD vector operations using AVX/SSE
; Author: Eddi Andreé Salazar Matos
; Windows x64 calling convention
;
; Vec2 layout: [x: f32, y: f32] = 8 bytes
; Process 4 Vec2s at a time with AVX (256-bit)

default rel
bits 64

section .text

; Batch add Vec2 arrays
; RCX = a (Vec2*), RDX = b (Vec2*), R8 = out (Vec2*), R9 = count
global simd_vec2_add_batch
simd_vec2_add_batch:
    test    r9, r9
    jz      .done
    
    ; Process 4 Vec2s at a time (32 bytes = 256 bits)
    mov     rax, r9
    shr     rax, 2          ; count / 4
    jz      .scalar_loop
    
.avx_loop:
    vmovups ymm0, [rcx]     ; Load 4 Vec2s from a
    vmovups ymm1, [rdx]     ; Load 4 Vec2s from b
    vaddps  ymm0, ymm0, ymm1 ; Add
    vmovups [r8], ymm0      ; Store result
    
    add     rcx, 32
    add     rdx, 32
    add     r8, 32
    dec     rax
    jnz     .avx_loop
    
    ; Handle remaining elements
    and     r9, 3           ; count % 4
    jz      .done
    
.scalar_loop:
    movsd   xmm0, [rcx]     ; Load Vec2 from a
    movsd   xmm1, [rdx]     ; Load Vec2 from b
    addps   xmm0, xmm1      ; Add
    movsd   [r8], xmm0      ; Store
    
    add     rcx, 8
    add     rdx, 8
    add     r8, 8
    dec     r9
    jnz     .scalar_loop
    
.done:
    vzeroupper
    ret

; Batch multiply Vec2 array by scalar
; RCX = a (Vec2*), XMM1 = scalar, R8 = out (Vec2*), R9 = count
global simd_vec2_mul_scalar_batch
simd_vec2_mul_scalar_batch:
    test    r9, r9
    jz      .done
    
    ; Broadcast scalar to all lanes
    vbroadcastss ymm1, xmm1
    
    mov     rax, r9
    shr     rax, 2
    jz      .scalar_loop
    
.avx_loop:
    vmovups ymm0, [rcx]
    vmulps  ymm0, ymm0, ymm1
    vmovups [r8], ymm0
    
    add     rcx, 32
    add     r8, 32
    dec     rax
    jnz     .avx_loop
    
    and     r9, 3
    jz      .done
    
.scalar_loop:
    movsd   xmm0, [rcx]
    mulps   xmm0, xmm1
    movsd   [r8], xmm0
    
    add     rcx, 8
    add     r8, 8
    dec     r9
    jnz     .scalar_loop
    
.done:
    vzeroupper
    ret

; Batch dot product
; RCX = a (Vec2*), RDX = b (Vec2*), R8 = out (f32*), R9 = count
global simd_dot_product_batch
simd_dot_product_batch:
    test    r9, r9
    jz      .done
    
.loop:
    movsd   xmm0, [rcx]     ; a.x, a.y
    movsd   xmm1, [rdx]     ; b.x, b.y
    mulps   xmm0, xmm1      ; a.x*b.x, a.y*b.y
    haddps  xmm0, xmm0      ; a.x*b.x + a.y*b.y
    movss   [r8], xmm0      ; Store result
    
    add     rcx, 8
    add     rdx, 8
    add     r8, 4
    dec     r9
    jnz     .loop
    
.done:
    ret

; Batch normalize Vec2
; RCX = a (Vec2*), RDX = out (Vec2*), R8 = count
global simd_normalize_batch
simd_normalize_batch:
    test    r8, r8
    jz      .done
    
.loop:
    movsd   xmm0, [rcx]     ; x, y
    movaps  xmm1, xmm0
    mulps   xmm1, xmm1      ; x², y²
    haddps  xmm1, xmm1      ; x² + y²
    sqrtss  xmm1, xmm1      ; length
    
    ; Check for zero length
    xorps   xmm2, xmm2
    comiss  xmm1, xmm2
    jz      .store_zero
    
    ; Normalize
    shufps  xmm1, xmm1, 0   ; Broadcast length
    divps   xmm0, xmm1
    movsd   [rdx], xmm0
    jmp     .next
    
.store_zero:
    movsd   [rdx], xmm2
    
.next:
    add     rcx, 8
    add     rdx, 8
    dec     r8
    jnz     .loop
    
.done:
    ret
