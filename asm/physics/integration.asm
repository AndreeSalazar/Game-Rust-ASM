; integration.asm - Physics integration (Semi-implicit Euler)
; Author: Eddi Andreé Salazar Matos
; Hot path for updating many bodies
;
; Body layout (SoA for SIMD):
;   positions: [x0, y0, x1, y1, ...] (f32 pairs)
;   velocities: [vx0, vy0, vx1, vy1, ...] (f32 pairs)
;   accelerations: [ax0, ay0, ax1, ay1, ...] (f32 pairs)

default rel
bits 64

section .text

; Batch physics integration (Semi-implicit Euler)
; RCX = positions (f32*), RDX = velocities (f32*), R8 = accelerations (f32*)
; R9 = inv_masses (f32*), XMM4 = dt, [rsp+40] = count
global physics_integrate_batch
physics_integrate_batch:
    push    rbx
    push    r12
    push    r13
    
    mov     r10, [rsp+64]   ; count (adjusted for pushes)
    test    r10, r10
    jz      .done
    
    ; Broadcast dt to all lanes
    vbroadcastss ymm4, xmm4
    
    ; Process 4 bodies at a time (8 floats = 32 bytes)
    mov     rax, r10
    shr     rax, 2          ; count / 4
    jz      .scalar_start
    
    xor     rbx, rbx        ; offset = 0
    
.avx_loop:
    ; Load accelerations
    vmovups ymm0, [r8 + rbx]        ; a (8 floats = 4 Vec2)
    
    ; Load velocities
    vmovups ymm1, [rdx + rbx]       ; v
    
    ; v += a * dt
    vfmadd231ps ymm1, ymm0, ymm4    ; v = v + a * dt
    
    ; Store updated velocities
    vmovups [rdx + rbx], ymm1
    
    ; Load positions
    vmovups ymm2, [rcx + rbx]       ; p
    
    ; p += v * dt
    vfmadd231ps ymm2, ymm1, ymm4    ; p = p + v * dt
    
    ; Store updated positions
    vmovups [rcx + rbx], ymm2
    
    ; Clear accelerations
    vxorps  ymm0, ymm0, ymm0
    vmovups [r8 + rbx], ymm0
    
    add     rbx, 32         ; Next 4 bodies
    dec     rax
    jnz     .avx_loop
    
.scalar_start:
    ; Handle remaining bodies
    and     r10, 3
    jz      .done
    
    ; Scalar loop for remaining
.scalar_loop:
    ; Load acceleration
    movsd   xmm0, [r8 + rbx]        ; a.x, a.y
    
    ; Load velocity
    movsd   xmm1, [rdx + rbx]       ; v.x, v.y
    
    ; v += a * dt
    movaps  xmm2, xmm0
    mulps   xmm2, xmm4
    addps   xmm1, xmm2
    
    ; Store velocity
    movsd   [rdx + rbx], xmm1
    
    ; Load position
    movsd   xmm2, [rcx + rbx]       ; p.x, p.y
    
    ; p += v * dt
    movaps  xmm3, xmm1
    mulps   xmm3, xmm4
    addps   xmm2, xmm3
    
    ; Store position
    movsd   [rcx + rbx], xmm2
    
    ; Clear acceleration
    xorps   xmm0, xmm0
    movsd   [r8 + rbx], xmm0
    
    add     rbx, 8
    dec     r10
    jnz     .scalar_loop
    
.done:
    vzeroupper
    pop     r13
    pop     r12
    pop     rbx
    ret

; Apply impulse to body
; RCX = velocity (Vec2*), XMM1 = impulse (Vec2), XMM2 = inv_mass
global apply_impulse
apply_impulse:
    movsd   xmm0, [rcx]     ; Load velocity
    shufps  xmm2, xmm2, 0   ; Broadcast inv_mass
    mulps   xmm1, xmm2      ; impulse * inv_mass
    addps   xmm0, xmm1      ; v += impulse * inv_mass
    movsd   [rcx], xmm0     ; Store velocity
    ret

; Verlet integration step
; RCX = positions (f32*), RDX = prev_positions (f32*), R8 = accelerations (f32*)
; XMM3 = dt², R9 = count
global verlet_integrate_batch
verlet_integrate_batch:
    test    r9, r9
    jz      .done
    
    vbroadcastss ymm3, xmm3 ; Broadcast dt²
    
    xor     rax, rax
    
.loop:
    ; Load current position
    movsd   xmm0, [rcx + rax]       ; p
    
    ; Load previous position
    movsd   xmm1, [rdx + rax]       ; p_prev
    
    ; Load acceleration
    movsd   xmm2, [r8 + rax]        ; a
    
    ; Calculate velocity (p - p_prev)
    movaps  xmm4, xmm0
    subps   xmm4, xmm1              ; v = p - p_prev
    
    ; Store current as previous
    movsd   [rdx + rax], xmm0
    
    ; new_p = p + v + a * dt²
    addps   xmm0, xmm4              ; p + v
    mulps   xmm2, xmm3              ; a * dt²
    addps   xmm0, xmm2              ; p + v + a * dt²
    
    ; Store new position
    movsd   [rcx + rax], xmm0
    
    ; Clear acceleration
    xorps   xmm2, xmm2
    movsd   [r8 + rax], xmm2
    
    add     rax, 8
    dec     r9
    jnz     .loop
    
.done:
    ret
