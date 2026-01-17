; timing.asm - High-precision timing using RDTSC
; Author: Eddi Andreé Salazar Matos
; Windows x64 calling convention
; 
; Exports:
;   rdtsc_start() -> u64
;   rdtsc_end() -> u64  
;   rdtsc_cycles_to_ns(cycles: u64, freq_mhz: u64) -> u64

default rel
bits 64

section .text

; Serialize and read timestamp counter (start measurement)
; Returns: RAX = timestamp
global rdtsc_start
rdtsc_start:
    ; Serialize to prevent out-of-order execution
    xor     eax, eax
    cpuid
    ; Read timestamp counter
    rdtsc
    ; Combine EDX:EAX into RAX
    shl     rdx, 32
    or      rax, rdx
    ret

; Read timestamp counter and serialize (end measurement)
; Returns: RAX = timestamp
global rdtsc_end
rdtsc_end:
    ; Read timestamp counter first
    rdtscp
    ; Combine EDX:EAX into RAX
    shl     rdx, 32
    or      rax, rdx
    ; Save result
    mov     rcx, rax
    ; Serialize
    xor     eax, eax
    cpuid
    ; Return saved result
    mov     rax, rcx
    ret

; Convert cycles to nanoseconds
; RCX = cycles, RDX = freq_mhz
; Returns: RAX = nanoseconds
global rdtsc_cycles_to_ns
rdtsc_cycles_to_ns:
    mov     rax, rcx        ; cycles
    mov     rcx, 1000       ; multiply by 1000 for ns
    mul     rcx             ; RDX:RAX = cycles * 1000
    div     rdx             ; RAX = (cycles * 1000) / freq_mhz
    ret

; Prefetch data into cache
; RCX = address
global prefetch_data
prefetch_data:
    prefetcht0 [rcx]
    ret

; Memory fence for timing accuracy
global memory_fence
memory_fence:
    mfence
    ret
