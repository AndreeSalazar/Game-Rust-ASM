//! Build Script - Game Engine X
//! Author: Eddi Andreé Salazar Matos
//! License: MIT
//!
//! Compiles NASM assembly files and links them with Rust.
//! Falls back to pure Rust if NASM is not available.

use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(no_asm)");
    println!("cargo:rerun-if-changed=asm/");
    
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    
    // Find NASM executable
    let nasm_path = match find_nasm() {
        Some(path) => {
            println!("cargo:warning=[NASM] Found at: {}", path);
            path
        }
        None => {
            println!("cargo:warning=[NASM] Not found - using Rust fallback");
            println!("cargo:rustc-cfg=no_asm");
            return;
        }
    };
    
    // Verify NASM works
    if !verify_nasm(&nasm_path) {
        println!("cargo:warning=[NASM] Verification failed - using Rust fallback");
        println!("cargo:rustc-cfg=no_asm");
        return;
    }
    
    // Set NASM environment for nasm-rs
    env::set_var("NASM", &nasm_path);
    
    // ASM source files
    let asm_files = [
        "asm/core/timing.asm",
        "asm/math/simd_vec.asm",
        "asm/math/fixed_point.asm",
        "asm/physics/collision.asm",
        "asm/physics/integration.asm",
        "asm/render/raycast.asm",
    ];
    
    // Check which files exist
    let mut existing_files = Vec::new();
    for asm_file in &asm_files {
        let full_path = manifest_dir.join(asm_file);
        if full_path.exists() {
            println!("cargo:rerun-if-changed={}", asm_file);
            existing_files.push(asm_file.to_string());
        } else {
            println!("cargo:warning=[ASM] File not found: {}", asm_file);
        }
    }
    
    if existing_files.is_empty() {
        println!("cargo:warning=[ASM] No ASM files found - using Rust fallback");
        println!("cargo:rustc-cfg=no_asm");
        return;
    }
    
    // Compile with nasm-rs
    let mut build = nasm_rs::Build::new();
    build.target("win64");
    
    for file in &existing_files {
        build.file(file);
    }
    
    match build.compile("asm_core") {
        Ok(_) => {
            println!("cargo:rustc-link-lib=static=asm_core");
            println!("cargo:rustc-link-search=native={}", out_dir.display());
            println!("cargo:warning=[ASM] Successfully compiled {} files", existing_files.len());
        }
        Err(e) => {
            println!("cargo:warning=[ASM] Compilation failed: {}", e);
            println!("cargo:warning=[ASM] Falling back to Rust implementations");
            println!("cargo:rustc-cfg=no_asm");
        }
    }
}

/// Find NASM executable on the system
fn find_nasm() -> Option<String> {
    // 1. Check NASM environment variable
    if let Ok(nasm) = env::var("NASM") {
        let path = PathBuf::from(&nasm);
        if path.exists() && path.is_file() {
            return Some(nasm);
        }
    }
    
    // 2. Check known Windows locations
    let known_paths = [
        // User-specific locations
        r"C:\Users\andre\AppData\Local\bin\NASM\nasm.exe",
        r"C:\Users\andre\AppData\Local\NASM\nasm.exe",
        r"C:\Users\andre\NASM\nasm.exe",
        // System-wide locations
        r"C:\NASM\nasm.exe",
        r"C:\Program Files\NASM\nasm.exe",
        r"C:\Program Files (x86)\NASM\nasm.exe",
        // Chocolatey
        r"C:\ProgramData\chocolatey\bin\nasm.exe",
        // Scoop
        r"C:\Users\andre\scoop\shims\nasm.exe",
    ];
    
    for path in &known_paths {
        let p = PathBuf::from(path);
        if p.exists() && p.is_file() {
            return Some(path.to_string());
        }
    }
    
    // 3. Search in PATH
    if let Ok(path_var) = env::var("PATH") {
        let separator = if cfg!(windows) { ';' } else { ':' };
        for dir in path_var.split(separator) {
            let nasm_exe = if cfg!(windows) { "nasm.exe" } else { "nasm" };
            let nasm_path = PathBuf::from(dir).join(nasm_exe);
            if nasm_path.exists() && nasm_path.is_file() {
                return Some(nasm_path.to_string_lossy().to_string());
            }
        }
    }
    
    // 4. Try 'where' command on Windows
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg("nasm").output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    let path = first_line.trim();
                    if !path.is_empty() && PathBuf::from(path).exists() {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }
    
    None
}

/// Verify NASM executable works correctly
fn verify_nasm(nasm_path: &str) -> bool {
    match Command::new(nasm_path).arg("-v").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                println!("cargo:warning=[NASM] Version: {}", version.trim());
                true
            } else {
                false
            }
        }
        Err(e) => {
            println!("cargo:warning=[NASM] Failed to execute: {}", e);
            false
        }
    }
}
