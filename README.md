# 🎮 Game Engine X

### Rust + ASM Deterministic 2D Game Engine

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![NASM](https://img.shields.io/badge/NASM-x64-blue.svg)](https://www.nasm.us/)

> **Autor:** Eddi Andreé Salazar Matos  
> **Licencia:** MIT

Motor de juegos 2D determinista de alto rendimiento con **Rust como controlador principal** y **ASM (NASM) para hot paths críticos**.

---

## 📐 Arquitectura

```
Game Logic (Rust)
├── src/
│   ├── core/          # Timing (RDTSC), game loop, profiler
│   ├── ecs/           # Entity Component System (hecs)
│   ├── math/          # Vec2, FixedPoint, SIMD batches
│   ├── physics/       # Collision, integration, broad phase
│   ├── render/        # Software renderer, raycaster
│   ├── input/         # Input handling
│   └── audio/         # Audio (placeholder)
│
├── asm/               # NASM assembly (hot paths only)
│   ├── core/timing.asm      # RDTSC nanosecond timing
│   ├── math/simd_vec.asm    # AVX/SSE vector operations
│   ├── math/fixed_point.asm # 16.16 fixed-point math
│   ├── physics/collision.asm # SIMD collision detection
│   ├── physics/integration.asm # Batch physics integration
│   └── render/raycast.asm   # DDA raycasting inner loop
│
└── games/             # Game implementations
    ├── physics_2d/    # Platformer / Bullet-hell
    ├── raycaster/     # DOOM-like engine
    ├── massive_sim/   # 10K+ entity simulation
    └── fighting/      # Frame-perfect fighter
```

## Regla de Oro

> **Rust decide, ASM ejecuta**

- ASM: Solo matemáticas en loops calientes
- ASM: Nunca lógica de juego
- ASM: Nunca expuesto directamente

## 🎮 Juegos Incluidos

### 1. Physics 2D (`cargo run --bin physics_2d --release`)
Platformer con físicas custom y colisiones.
- **Controles:** WASD/Flechas para mover, Espacio para saltar, ESC para salir
- Plataformas flotantes y pelotas rebotando
- Fixed timestep determinista a 60 FPS

### 2. Raycaster (`cargo run --bin raycaster --release`)
Engine tipo DOOM/Wolfenstein con software rendering.
- **Controles:** WASD/Flechas para mover/rotar, ESC para salir
- DDA raycasting algorithm
- Minimap integrado en esquina superior

### 3. Massive Sim (`cargo run --bin massive_sim --release`)
Simulación de 5,000+ entidades a 60 FPS.
- **Controles:** ESC para salir
- Structure of Arrays (SoA) para cache efficiency
- SIMD-ready batch updates

### 4. Fighting (`cargo run --bin fighting --release`)
Juego de pelea 2D con frame-perfect input.
- **P1:** WASD mover, F puño, G patada
- **P2:** Flechas mover, K puño, L patada
- Hitbox/hurtbox collision system

## Build

### Requisitos
- Rust 1.70+
- NASM (opcional, fallback a Rust si no disponible)

### Compilar
```bash
# Debug
cargo build

# Release (optimizado)
cargo build --release

# Ejecutar juego específico
cargo run --bin physics_2d --release
cargo run --bin raycaster --release
cargo run --bin massive_sim --release
cargo run --bin fighting --release
```

### NASM
El build.rs busca NASM automáticamente en:
- `C:\Users\andre\AppData\Local\bin\NASM\nasm.exe`
- `C:\NASM\nasm.exe`
- `C:\Program Files\NASM\nasm.exe`
- PATH del sistema

Si NASM no está disponible, el engine usa implementaciones Rust como fallback.

## Stack Técnico

- **Rust**: Core engine, ECS, scheduler, APIs
- **winit 0.30**: Window + input
- **softbuffer**: Software rendering (sin GPU)
- **hecs**: ECS ligero
- **NASM**: Assembly x64 (Windows)
- **SIMD**: AVX2 / SSE2 (en archivos ASM)

## Qué Demuestra Este Proyecto

| Skill | Implementación |
|-------|----------------|
| ECS | Sistema propio con hecs |
| Fixed Timestep | Game loop determinista |
| Physics SIMD | Colisiones batch en ASM |
| Cache-friendly | SoA para simulación masiva |
| Low-level | RDTSC timing, fixed-point math |
| Software Rendering | Raycaster sin GPU |

## 👨‍💻 Roles que Aplica

- **Engine Programmer** - Arquitectura de motor completa
- **Systems Programmer** - Integración Rust + ASM
- **Gameplay Systems Engineer** - ECS, física, input
- **Performance Engineer** - SIMD, cache optimization

---

## 📄 Licencia

Este proyecto está licenciado bajo la **Licencia MIT** - ver el archivo [LICENSE](LICENSE) para más detalles.

```
MIT License
Copyright (c) 2026 Eddi Andreé Salazar Matos
```

---

## 🙏 Créditos

**Desarrollado por:** Eddi Andreé Salazar Matos

### Tecnologías Utilizadas
- [Rust](https://www.rust-lang.org/) - Lenguaje principal
- [NASM](https://www.nasm.us/) - Ensamblador x64
- [hecs](https://crates.io/crates/hecs) - ECS ligero
- [winit](https://crates.io/crates/winit) - Windowing
- [pixels](https://crates.io/crates/pixels) - Software rendering

---

<p align="center">
  <b>Game Engine X</b> - Motor de juegos determinista Rust + ASM<br>
  Hecho con ❤️ por Eddi Andreé Salazar Matos
</p>
