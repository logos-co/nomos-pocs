use smallvec::SmallVec;

#[derive(Debug, Clone)]
pub struct OperationBuffer {
    pub read_buffer: Box<[u8]>,
    pub write_buffer: Box<[u8]>,
    pub key_buffer: SmallVec<[u8; 64]>,
}

impl OperationBuffer {
    #[must_use]
    pub fn new(read_size: usize, write_size: usize) -> Self {
        Self {
            read_buffer: vec![0u8; read_size].into_boxed_slice(),
            write_buffer: vec![0u8; write_size].into_boxed_slice(),
            key_buffer: SmallVec::new(),
        }
    }

    #[must_use]
    pub fn read_slice(&self) -> &[u8] {
        &self.read_buffer
    }

    pub fn write_slice_mut(&mut self) -> &mut [u8] {
        &mut self.write_buffer
    }

    pub fn prepare_key<T: AsRef<[u8]>>(&mut self, key_data: T) -> &[u8] {
        let key_bytes = key_data.as_ref();

        self.key_buffer.clear();
        self.key_buffer.extend_from_slice(key_bytes);

        &self.key_buffer
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TimingMeasurement {
    pub start_time: std::time::Instant,
}

impl TimingMeasurement {
    #[inline]
    #[must_use]
    pub fn start() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }

    #[inline]
    #[must_use]
    pub fn end(self) -> std::time::Duration {
        self.start_time.elapsed()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ThreadLocalMetrics {
    pub operations_count: u64,
    pub success_count: u64,
    pub error_count: u64,
    pub bytes_processed: u64,
    pub latency_sum_micros: u64,
    pub latency_count: u64,
}

impl ThreadLocalMetrics {
    #[inline]
    pub const fn record_operation(
        &mut self,
        success: bool,
        bytes: u64,
        latency: std::time::Duration,
    ) {
        self.operations_count += 1;

        if success {
            self.success_count += 1;
        } else {
            self.error_count += 1;
        }

        self.bytes_processed += bytes;
        self.latency_sum_micros += latency.as_micros() as u64;
        self.latency_count += 1;
    }

    #[must_use]
    pub fn average_latency_micros(&self) -> f64 {
        if self.latency_count > 0 {
            self.latency_sum_micros as f64 / self.latency_count as f64
        } else {
            0.0
        }
    }

    pub const fn fold_into(self, global: &mut Self) {
        global.operations_count += self.operations_count;
        global.success_count += self.success_count;
        global.error_count += self.error_count;
        global.bytes_processed += self.bytes_processed;
        global.latency_sum_micros += self.latency_sum_micros;
        global.latency_count += self.latency_count;
    }
}

pub trait EfficientIteratorExt: Iterator {
    fn collect_presized(self, size_hint: usize) -> Vec<Self::Item>
    where
        Self: Sized,
    {
        let mut vec = Vec::with_capacity(size_hint);
        vec.extend(self);
        vec
    }

    fn collect_small_8(self) -> SmallVec<[Self::Item; 8]>
    where
        Self: Sized,
    {
        let mut vec: SmallVec<[Self::Item; 8]> = SmallVec::new();
        vec.extend(self);
        vec
    }
}

impl<I: Iterator> EfficientIteratorExt for I {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::benchmark::utilities::create_blob_id;

    #[test]
    fn test_operation_buffer_efficiency() {
        let mut buffer = OperationBuffer::new(1024, 2048);

        let key1 = buffer.prepare_key(b"test_key_1");
        assert_eq!(key1, b"test_key_1");

        let key2 = buffer.prepare_key(b"different_key");
        assert_eq!(key2, b"different_key");

        assert!(buffer.key_buffer.capacity() >= 12);
    }

    #[test]
    fn test_efficient_id_creation() {
        let header_id = create_header_id_efficient(12345);
        let blob_id = create_blob_id(100, 5);

        assert_ne!(header_id.as_ref(), &[0u8; 32]);
        assert_ne!(blob_id.as_ref(), &[0u8; 32]);
    }

    #[test]
    fn test_thread_local_metrics() {
        let mut metrics = ThreadLocalMetrics::default();

        metrics.record_operation(true, 1024, std::time::Duration::from_micros(500));
        metrics.record_operation(false, 0, std::time::Duration::from_micros(1000));

        assert_eq!(metrics.operations_count, 2);
        assert_eq!(metrics.success_count, 1);
        assert_eq!(metrics.error_count, 1);
        assert_eq!(metrics.bytes_processed, 1024);
        assert_eq!(metrics.average_latency_micros(), 750.0);
    }
}
