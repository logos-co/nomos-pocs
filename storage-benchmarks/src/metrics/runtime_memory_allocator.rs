use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

pub struct RuntimeValidatorAllocator {
    inner: System,
    allocated: AtomicUsize,
    limit: AtomicUsize,
}

impl RuntimeValidatorAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocated: AtomicUsize::new(0),
            limit: AtomicUsize::new(16 * 1024 * 1024 * 1024),
        }
    }

    pub fn set_limit_gb(&self, limit_gb: usize) {
        let limit_bytes = limit_gb * 1024 * 1024 * 1024;
        self.limit.store(limit_bytes, Ordering::SeqCst);
        log::info!(
            "Memory limit updated to {}GB ({} bytes)",
            limit_gb,
            limit_bytes
        );
    }

    pub fn usage_mb(&self) -> f64 {
        self.allocated.load(Ordering::Relaxed) as f64 / 1024.0 / 1024.0
    }

    pub fn usage_percent(&self) -> f64 {
        let current = self.allocated.load(Ordering::Relaxed);
        let limit = self.limit.load(Ordering::Relaxed);
        if limit > 0 {
            current as f64 / limit as f64 * 100.0
        } else {
            0.0
        }
    }

    pub fn limit_gb(&self) -> usize {
        self.limit.load(Ordering::Relaxed) / 1024 / 1024 / 1024
    }

    pub fn actual_limit_gb(&self) -> usize {
        self.limit_gb()
    }

    pub fn would_exceed_limit(&self, size: usize) -> bool {
        let current = self.allocated.load(Ordering::Relaxed);
        let limit = self.limit.load(Ordering::Relaxed);
        current + size > limit
    }

    pub fn allocation_failures(&self) -> u64 {
        0
    }
}

unsafe impl GlobalAlloc for RuntimeValidatorAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        let current = self.allocated.fetch_add(size, Ordering::SeqCst);
        let limit = self.limit.load(Ordering::Relaxed);

        if current + size > limit {
            self.allocated.fetch_sub(size, Ordering::SeqCst);

            if size >= 1024 * 1024 {
                log::warn!(
                    "Memory limit exceeded: {}MB allocation blocked ({}GB limit, {:.1}% used)",
                    size / 1024 / 1024,
                    limit / 1024 / 1024 / 1024,
                    current as f64 / limit as f64 * 100.0
                );
            }

            return std::ptr::null_mut();
        }

        unsafe { self.inner.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocated.fetch_sub(layout.size(), Ordering::SeqCst);

        unsafe {
            self.inner.dealloc(ptr, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_limit_setting() {
        let allocator = RuntimeValidatorAllocator::new();

        assert_eq!(allocator.limit_gb(), 16);

        allocator.set_limit_gb(8);
        assert_eq!(allocator.limit_gb(), 8);
        assert_eq!(allocator.actual_limit_gb(), 8);

        allocator.set_limit_gb(32);
        assert_eq!(allocator.limit_gb(), 32);
    }
}
