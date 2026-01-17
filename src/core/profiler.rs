//! Real-time profiler for performance monitoring

use std::collections::HashMap;
use super::timing::Timer;

/// Performance profiler
pub struct Profiler {
    samples: HashMap<&'static str, ProfileSample>,
    timer: Timer,
}

#[derive(Default, Clone)]
pub struct ProfileSample {
    pub total_ns: u64,
    pub count: u64,
    pub min_ns: u64,
    pub max_ns: u64,
}

impl ProfileSample {
    pub fn avg_ms(&self) -> f64 {
        if self.count > 0 {
            (self.total_ns as f64 / self.count as f64) / 1_000_000.0
        } else {
            0.0
        }
    }
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            samples: HashMap::new(),
            timer: Timer::new(),
        }
    }
    
    /// Record a sample
    pub fn record(&mut self, name: &'static str, ns: u64) {
        let sample = self.samples.entry(name).or_insert(ProfileSample {
            total_ns: 0,
            count: 0,
            min_ns: u64::MAX,
            max_ns: 0,
        });
        
        sample.total_ns += ns;
        sample.count += 1;
        sample.min_ns = sample.min_ns.min(ns);
        sample.max_ns = sample.max_ns.max(ns);
    }
    
    /// Get sample by name
    pub fn get(&self, name: &str) -> Option<&ProfileSample> {
        self.samples.get(name)
    }
    
    /// Reset all samples
    pub fn reset(&mut self) {
        self.samples.clear();
    }
    
    /// Print summary
    pub fn print_summary(&self) {
        println!("\n=== Profiler Summary ===");
        for (name, sample) in &self.samples {
            println!(
                "{}: avg={:.3}ms, min={:.3}ms, max={:.3}ms, count={}",
                name,
                sample.avg_ms(),
                sample.min_ns as f64 / 1_000_000.0,
                sample.max_ns as f64 / 1_000_000.0,
                sample.count
            );
        }
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
