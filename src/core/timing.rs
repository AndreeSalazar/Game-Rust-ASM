//! High-precision timing using ASM RDTSC
//! 
//! ASM provides nanosecond-precision timing via RDTSC instruction.
//! Rust handles all logic, ASM only reads the timestamp counter.

#[cfg(not(no_asm))]
extern "C" {
    fn rdtsc_start() -> u64;
    fn rdtsc_end() -> u64;
    fn rdtsc_cycles_to_ns(cycles: u64, freq_mhz: u64) -> u64;
}

/// High-precision timer using RDTSC
pub struct Timer {
    start_cycles: u64,
    frequency_mhz: u64,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start_cycles: 0,
            frequency_mhz: Self::estimate_frequency(),
        }
    }
    
    /// Start timing
    #[inline]
    pub fn start(&mut self) {
        #[cfg(not(no_asm))]
        {
            self.start_cycles = unsafe { rdtsc_start() };
        }
        #[cfg(no_asm)]
        {
            self.start_cycles = Self::rdtsc_fallback();
        }
    }
    
    /// Get elapsed nanoseconds since start
    #[inline]
    pub fn elapsed_ns(&self) -> u64 {
        let end_cycles;
        
        #[cfg(not(no_asm))]
        {
            end_cycles = unsafe { rdtsc_end() };
        }
        #[cfg(no_asm)]
        {
            end_cycles = Self::rdtsc_fallback();
        }
        
        let cycles = end_cycles.saturating_sub(self.start_cycles);
        self.cycles_to_ns(cycles)
    }
    
    /// Get elapsed microseconds
    #[inline]
    pub fn elapsed_us(&self) -> u64 {
        self.elapsed_ns() / 1000
    }
    
    /// Get elapsed milliseconds
    #[inline]
    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed_ns() as f64 / 1_000_000.0
    }
    
    #[inline]
    fn cycles_to_ns(&self, cycles: u64) -> u64 {
        #[cfg(not(no_asm))]
        {
            unsafe { rdtsc_cycles_to_ns(cycles, self.frequency_mhz) }
        }
        #[cfg(no_asm)]
        {
            (cycles * 1000) / self.frequency_mhz
        }
    }
    
    /// Estimate CPU frequency in MHz
    fn estimate_frequency() -> u64 {
        use std::time::{Duration, Instant};
        
        let start_time = Instant::now();
        let start_cycles = Self::rdtsc_fallback();
        
        std::thread::sleep(Duration::from_millis(10));
        
        let end_cycles = Self::rdtsc_fallback();
        let elapsed = start_time.elapsed();
        
        let cycles = end_cycles.saturating_sub(start_cycles);
        let ns = elapsed.as_nanos() as u64;
        
        if ns > 0 {
            (cycles * 1000) / ns
        } else {
            3000 // Default 3GHz
        }
    }
    
    /// Rust fallback for RDTSC
    #[inline]
    fn rdtsc_fallback() -> u64 {
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                std::arch::x86_64::_rdtsc()
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            std::time::Instant::now().elapsed().as_nanos() as u64
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// Scope-based timing for profiling
pub struct ScopedTimer<'a> {
    name: &'a str,
    timer: Timer,
}

impl<'a> ScopedTimer<'a> {
    pub fn new(name: &'a str) -> Self {
        let mut timer = Timer::new();
        timer.start();
        Self { name, timer }
    }
}

impl<'a> Drop for ScopedTimer<'a> {
    fn drop(&mut self) {
        log::trace!("{}: {:.3}ms", self.name, self.timer.elapsed_ms());
    }
}

/// Macro for easy scope timing
#[macro_export]
macro_rules! time_scope {
    ($name:expr) => {
        let _timer = $crate::core::timing::ScopedTimer::new($name);
    };
}
