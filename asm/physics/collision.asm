; collision.asm - SIMD collision detection
; Author: Eddi Andreé Salazar Matos
; Hot path for narrow phase collision
;
; AABB: [min_x, min_y, max_x, max_y] = 16 bytes
; Circle: [center_x, center_y, radius, _padding] = 16 bytes

default rel
bits 64

section .text

; Batch AABB vs AABB test
; RCX = positions (Vec2*), RDX = half_extents (Vec2*), R8 = count
; R9 = pairs output (u32*), [rsp+40] = max_pairs
; Returns: RAX = number of colliding pairs
global collision_aabb_batch
collision_aabb_batch:
    push    rbx
    push    rsi
    push    rdi
    push    r12
    push    r13
    push    r14
    push    r15
    
    mov     r12, rcx        ; positions
    mov     r13, rdx        ; half_extents
    mov     r14, r8         ; count
    mov     r15, r9         ; pairs output
    mov     rbx, [rsp+96]   ; max_pairs (adjusted for pushes)
    
    xor     rsi, rsi        ; pair count = 0
    xor     rdi, rdi        ; i = 0
    
.outer_loop:
    cmp     rdi, r14
    jge     .done
    
    ; Load AABB i
    mov     rax, rdi
    shl     rax, 3          ; * 8 (Vec2 size)
    movsd   xmm0, [r12 + rax]       ; pos_i
    movsd   xmm1, [r13 + rax]       ; half_i
    
    ; Calculate min/max for i
    movaps  xmm2, xmm0
    subps   xmm2, xmm1      ; min_i = pos - half
    movaps  xmm3, xmm0
    addps   xmm3, xmm1      ; max_i = pos + half
    
    lea     rcx, [rdi + 1]  ; j = i + 1
    
.inner_loop:
    cmp     rcx, r14
    jge     .next_outer
    cmp     rsi, rbx        ; Check max_pairs
    jge     .done
    
    ; Load AABB j
    mov     rax, rcx
    shl     rax, 3
    movsd   xmm4, [r12 + rax]       ; pos_j
    movsd   xmm5, [r13 + rax]       ; half_j
    
    ; Calculate min/max for j
    movaps  xmm6, xmm4
    subps   xmm6, xmm5      ; min_j
    movaps  xmm7, xmm4
    addps   xmm7, xmm5      ; max_j
    
    ; AABB intersection test
    ; overlap = (min_i <= max_j) && (max_i >= min_j)
    movaps  xmm0, xmm2      ; min_i
    cmpps   xmm0, xmm7, 2   ; min_i <= max_j (LE)
    movaps  xmm1, xmm3      ; max_i
    cmpps   xmm1, xmm6, 5   ; max_i >= min_j (NLT)
    andps   xmm0, xmm1      ; Both conditions
    
    ; Check both x and y overlap
    movmskps eax, xmm0
    and     eax, 3          ; Only care about x and y
    cmp     eax, 3
    jne     .no_collision
    
    ; Store collision pair
    mov     rax, rsi
    shl     rax, 3          ; * 8 (pair = 2 * u32)
    mov     [r15 + rax], edi
    mov     [r15 + rax + 4], ecx
    inc     rsi
    
.no_collision:
    inc     rcx
    jmp     .inner_loop
    
.next_outer:
    inc     rdi
    jmp     .outer_loop
    
.done:
    mov     rax, rsi        ; Return pair count
    
    pop     r15
    pop     r14
    pop     r13
    pop     r12
    pop     rdi
    pop     rsi
    pop     rbx
    ret

; Batch circle vs circle test
; RCX = positions (Vec2*), RDX = radii (f32*), R8 = count
; R9 = pairs output (u32*), [rsp+40] = max_pairs
; Returns: RAX = number of colliding pairs
global collision_circle_batch
collision_circle_batch:
    push    rbx
    push    rsi
    push    rdi
    push    r12
    push    r13
    push    r14
    push    r15
    
    mov     r12, rcx        ; positions
    mov     r13, rdx        ; radii
    mov     r14, r8         ; count
    mov     r15, r9         ; pairs output
    mov     rbx, [rsp+96]   ; max_pairs
    
    xor     rsi, rsi        ; pair count
    xor     rdi, rdi        ; i
    
.outer_loop:
    cmp     rdi, r14
    jge     .done
    
    ; Load circle i
    mov     rax, rdi
    shl     rax, 3
    movsd   xmm0, [r12 + rax]       ; pos_i (x, y)
    mov     rax, rdi
    shl     rax, 2
    movss   xmm1, [r13 + rax]       ; radius_i
    
    lea     rcx, [rdi + 1]
    
.inner_loop:
    cmp     rcx, r14
    jge     .next_outer
    cmp     rsi, rbx
    jge     .done
    
    ; Load circle j
    mov     rax, rcx
    shl     rax, 3
    movsd   xmm2, [r12 + rax]       ; pos_j
    mov     rax, rcx
    shl     rax, 2
    movss   xmm3, [r13 + rax]       ; radius_j
    
    ; Calculate distance squared
    movaps  xmm4, xmm2
    subps   xmm4, xmm0      ; diff = pos_j - pos_i
    mulps   xmm4, xmm4      ; diff²
    haddps  xmm4, xmm4      ; dist² = diff.x² + diff.y²
    
    ; Calculate (r_i + r_j)²
    addss   xmm1, xmm3      ; r_sum = r_i + r_j
    mulss   xmm1, xmm1      ; r_sum²
    
    ; Compare
    comiss  xmm4, xmm1
    jae     .no_collision
    
    ; Store pair
    mov     rax, rsi
    shl     rax, 3
    mov     [r15 + rax], edi
    mov     [r15 + rax + 4], ecx
    inc     rsi
    
.no_collision:
    ; Reload radius_i for next iteration
    mov     rax, rdi
    shl     rax, 2
    movss   xmm1, [r13 + rax]
    
    inc     rcx
    jmp     .inner_loop
    
.next_outer:
    inc     rdi
    jmp     .outer_loop
    
.done:
    mov     rax, rsi
    
    pop     r15
    pop     r14
    pop     r13
    pop     r12
    pop     rdi
    pop     rsi
    pop     rbx
    ret
