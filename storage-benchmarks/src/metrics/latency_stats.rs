use std::time::Instant;

use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyPercentiles {
    pub p50_ms: f64,

    pub p90_ms: f64,

    pub p95_ms: f64,

    pub p99_ms: f64,

    pub max_ms: f64,

    pub mean_ms: f64,

    pub sample_count: u64,
}

pub struct LatencyTracker {
    histogram: Histogram<u64>,
    operation_count: u64,
}

impl Default for LatencyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl LatencyTracker {
    #[must_use]
    pub fn new() -> Self {
        Self {
            histogram: Histogram::new_with_bounds(1, 3_600_000_000, 3)
                .expect("Valid histogram bounds"),
            operation_count: 0,
        }
    }

    pub async fn record_async_operation<F, Fut, R>(&mut self, operation: F) -> R
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let start = Instant::now();
        let result = operation().await;
        let latency = start.elapsed();

        let latency_micros = latency.as_micros() as u64;
        if self.histogram.record(latency_micros).is_ok() {
            self.operation_count += 1;
        }

        result
    }

    #[must_use]
    pub fn get_percentiles(&self) -> LatencyPercentiles {
        if self.operation_count == 0 {
            return LatencyPercentiles {
                p50_ms: 0.0,
                p90_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                max_ms: 0.0,
                mean_ms: 0.0,
                sample_count: 0,
            };
        }

        let to_ms = |v: u64| v as f64 / 1000.0;

        LatencyPercentiles {
            p50_ms: to_ms(self.histogram.value_at_quantile(0.50)),
            p90_ms: to_ms(self.histogram.value_at_quantile(0.90)),
            p95_ms: to_ms(self.histogram.value_at_quantile(0.95)),
            p99_ms: to_ms(self.histogram.value_at_quantile(0.99)),
            max_ms: to_ms(self.histogram.max()),
            mean_ms: self.histogram.mean() / 1000.0,
            sample_count: self.operation_count,
        }
    }
}
