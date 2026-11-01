//! `ChunkBatchSizeCalculator`: how fast the client takes chunks, which it reports to the server
//! after every batch so the server paces its chunk sending.

use std::time::Instant;

const MAX_OLD_SAMPLES_WEIGHT: u32 = 49;
const CLAMP_COEFFICIENT: f64 = 3.0;

/// The running estimate of the time one chunk takes, from the batches seen so far.
#[derive(Debug)]
pub(super) struct ChunkBatchSizeCalculator {
    aggregated_nanos_per_chunk: f64,
    old_samples_weight: u32,
    batch_start: Instant,
}

impl ChunkBatchSizeCalculator {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            aggregated_nanos_per_chunk: 2_000_000.0,
            old_samples_weight: 1,
            batch_start: now,
        }
    }

    /// `onBatchStart`.
    pub(super) fn on_batch_start(&mut self, now: Instant) {
        self.batch_start = now;
    }

    /// `onBatchFinished`: an empty batch teaches nothing.
    pub(super) fn on_batch_finished(&mut self, batch_size: i32, now: Instant) {
        if batch_size > 0 {
            let duration = now.saturating_duration_since(self.batch_start).as_nanos() as f64;
            self.add_sample(duration / f64::from(batch_size));
        }
    }

    fn add_sample(&mut self, nanos_per_chunk: f64) {
        let clamped = nanos_per_chunk.clamp(
            self.aggregated_nanos_per_chunk / CLAMP_COEFFICIENT,
            self.aggregated_nanos_per_chunk * CLAMP_COEFFICIENT,
        );
        let weight = f64::from(self.old_samples_weight);
        self.aggregated_nanos_per_chunk =
            (self.aggregated_nanos_per_chunk * weight + clamped) / (weight + 1.0);
        self.old_samples_weight = (self.old_samples_weight + 1).min(MAX_OLD_SAMPLES_WEIGHT);
    }

    /// `getDesiredChunksPerTick`.
    pub(super) fn desired_chunks_per_tick(&self) -> f32 {
        (7_000_000.0 / self.aggregated_nanos_per_chunk) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_the_fresh_estimate() {
        let calculator = ChunkBatchSizeCalculator::new(Instant::now());
        assert_eq!(calculator.desired_chunks_per_tick(), 3.5);
    }

    #[test]
    fn a_fast_sample_is_clamped_to_a_third_and_averaged() {
        let mut calculator = ChunkBatchSizeCalculator::new(Instant::now());
        calculator.add_sample(1.0);
        // clamp(1, 2e6/3, 6e6) = 666666.67; (2e6 * 1 + 666666.67) / 2 = 1333333.33
        assert!((calculator.desired_chunks_per_tick() - 5.25).abs() < 1e-5);
    }

    #[test]
    fn a_slow_sample_is_clamped_to_three_times() {
        let mut calculator = ChunkBatchSizeCalculator::new(Instant::now());
        calculator.add_sample(1.0e12);
        // (2e6 + 6e6) / 2 = 4e6
        assert_eq!(calculator.desired_chunks_per_tick(), 1.75);
    }

    #[test]
    fn the_weight_stops_growing_at_49() {
        let mut calculator = ChunkBatchSizeCalculator::new(Instant::now());
        for _ in 0..100 {
            calculator.add_sample(2_000_000.0);
        }
        assert_eq!(calculator.old_samples_weight, 49);
    }

    #[test]
    fn an_empty_batch_changes_nothing() {
        let start = Instant::now();
        let mut calculator = ChunkBatchSizeCalculator::new(start);
        calculator.on_batch_finished(0, start + std::time::Duration::from_secs(5));
        assert_eq!(calculator.desired_chunks_per_tick(), 3.5);
    }
}
