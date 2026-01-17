; raycast.asm - DDA raycasting inner loop
; Author: Eddi Andreé Salazar Matos
; Hot path for DOOM-like rendering
;
; This is the most performance-critical part of a raycaster

default rel
bits 64

section .data
    align 16
    max_dist: dd 20.0, 20.0, 20.0, 20.0

section .text

; Batch DDA raycasting
; RCX = pos_x (f32), RDX = pos_y (f32) - passed in XMM registers on Windows
; Actually: XMM0 = pos_x, XMM1 = pos_y
; R8 = dir_x (f32*), R9 = dir_y (f32*)
; [rsp+40] = map (u8*), [rsp+48] = map_width, [rsp+56] = map_height
; [rsp+64] = distances (f32*), [rsp+72] = hit_sides (u8*), [rsp+80] = count
global raycast_dda_batch
raycast_dda_batch:
    push    rbx
    push    rsi
    push    rdi
    push    r12
    push    r13
    push    r14
    push    r15
    sub     rsp, 32         ; Shadow space
    
    ; Save parameters
    movss   [rsp], xmm0     ; pos_x
    movss   [rsp+4], xmm1   ; pos_y
    mov     r12, r8         ; dir_x array
    mov     r13, r9         ; dir_y array
    
    ; Load stack parameters (adjusted for pushes + shadow)
    mov     r14, [rsp+32+56+40]     ; map
    mov     r15d, [rsp+32+56+48]    ; map_width
    mov     ebx, [rsp+32+56+56]     ; map_height
    mov     rsi, [rsp+32+56+64]     ; distances
    mov     rdi, [rsp+32+56+72]     ; hit_sides
    mov     ecx, [rsp+32+56+80]     ; count
    
    test    ecx, ecx
    jz      .done
    
    xor     eax, eax        ; ray index
    
.ray_loop:
    push    rax
    push    rcx
    
    ; Load ray direction
    mov     rcx, rax
    shl     rcx, 2          ; * 4 (f32 size)
    movss   xmm2, [r12 + rcx]       ; dir_x
    movss   xmm3, [r13 + rcx]       ; dir_y
    
    ; Load position
    movss   xmm0, [rsp+16]  ; pos_x (adjusted for pushes)
    movss   xmm1, [rsp+20]  ; pos_y
    
    ; Calculate map position
    cvttss2si r8d, xmm0     ; map_x = (int)pos_x
    cvttss2si r9d, xmm1     ; map_y = (int)pos_y
    
    ; Calculate delta distances
    ; delta_dist_x = abs(1 / dir_x)
    movss   xmm4, xmm2
    rcpss   xmm4, xmm4      ; 1 / dir_x (approximate)
    andps   xmm4, [rel abs_mask]    ; abs
    
    movss   xmm5, xmm3
    rcpss   xmm5, xmm5      ; 1 / dir_y
    andps   xmm5, [rel abs_mask]
    
    ; Calculate step and initial side_dist
    xor     r10d, r10d      ; step_x
    xor     r11d, r11d      ; step_y
    
    ; step_x and side_dist_x
    xorps   xmm6, xmm6
    comiss  xmm2, xmm6
    jae     .pos_dir_x
    
    ; Negative dir_x
    mov     r10d, -1
    cvtsi2ss xmm6, r8d      ; (float)map_x
    subss   xmm6, xmm0      ; map_x - pos_x
    addss   xmm6, [rel one] ; + 1? No, just (pos_x - map_x)
    ; Actually: side_dist_x = (pos_x - map_x) * delta_dist_x
    movss   xmm6, xmm0
    cvtsi2ss xmm7, r8d
    subss   xmm6, xmm7      ; pos_x - map_x
    mulss   xmm6, xmm4      ; * delta_dist_x
    jmp     .calc_step_y
    
.pos_dir_x:
    mov     r10d, 1
    cvtsi2ss xmm6, r8d
    addss   xmm6, [rel one] ; map_x + 1
    subss   xmm6, xmm0      ; (map_x + 1) - pos_x
    mulss   xmm6, xmm4      ; * delta_dist_x
    
.calc_step_y:
    comiss  xmm3, [rel zero]
    jae     .pos_dir_y
    
    mov     r11d, -1
    movss   xmm7, xmm1
    cvtsi2ss xmm8, r9d
    subss   xmm7, xmm8
    mulss   xmm7, xmm5
    jmp     .dda_loop
    
.pos_dir_y:
    mov     r11d, 1
    cvtsi2ss xmm7, r9d
    addss   xmm7, [rel one]
    subss   xmm7, xmm1
    mulss   xmm7, xmm5
    
.dda_loop:
    mov     ecx, 64         ; Max iterations
    xor     edx, edx        ; side = 0
    
.dda_step:
    dec     ecx
    jz      .max_dist_reached
    
    ; Choose side
    comiss  xmm6, xmm7
    jae     .step_y
    
    ; Step X
    addss   xmm6, xmm4
    add     r8d, r10d
    xor     edx, edx        ; side = 0
    jmp     .check_hit
    
.step_y:
    addss   xmm7, xmm5
    add     r9d, r11d
    mov     edx, 1          ; side = 1
    
.check_hit:
    ; Bounds check
    cmp     r8d, 0
    jl      .max_dist_reached
    cmp     r8d, r15d
    jge     .max_dist_reached
    cmp     r9d, 0
    jl      .max_dist_reached
    cmp     r9d, ebx
    jge     .max_dist_reached
    
    ; Map lookup
    mov     eax, r9d
    imul    eax, r15d       ; y * width
    add     eax, r8d        ; + x
    movzx   eax, byte [r14 + rax]
    test    eax, eax
    jz      .dda_step       ; No hit, continue
    
    ; Hit! Calculate distance
    test    edx, edx
    jnz     .calc_dist_y
    
    ; Distance for X side
    subss   xmm6, xmm4
    movss   xmm0, xmm6
    jmp     .store_result
    
.calc_dist_y:
    subss   xmm7, xmm5
    movss   xmm0, xmm7
    jmp     .store_result
    
.max_dist_reached:
    movss   xmm0, [rel max_dist]
    
.store_result:
    pop     rcx
    pop     rax
    
    ; Store distance and side
    mov     r8, rax
    shl     r8, 2
    movss   [rsi + r8], xmm0
    mov     [rdi + rax], dl
    
    inc     eax
    dec     ecx
    jnz     .ray_loop
    
.done:
    add     rsp, 32
    pop     r15
    pop     r14
    pop     r13
    pop     r12
    pop     rdi
    pop     rsi
    pop     rbx
    ret

section .rodata
    align 16
    abs_mask: dd 0x7FFFFFFF, 0x7FFFFFFF, 0x7FFFFFFF, 0x7FFFFFFF
    one: dd 1.0
    zero: dd 0.0
