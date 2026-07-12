use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::analyzer::{analyze_trajectory, TrajectoryEntry};
use crate::render_policy;

const DEFAULT_MAX_BUFFER: usize = 100;
const DEFAULT_MIN_ANALYSIS: usize = 10;
const DEFAULT_FLUSH_INTERVAL: Duration = Duration::from_secs(300);

/// Callback invoked with the YAML policy string after each flush.
pub type PolicyCallback = Box<dyn Fn(&str) + Send + 'static>;

/// Accumulates tool calls and periodically generates policies.
pub struct TrajectoryCollector {
    buffer: VecDeque<TrajectoryEntry>,
    max_buffer: usize,
    min_for_analysis: usize,
    last_flush: Instant,
    flush_interval: Duration,
    policy_path: Option<PathBuf>,
    on_policy: Option<PolicyCallback>,
    pub total_recorded: usize,
    pub total_flushes: usize,
    pub last_policy: Option<String>,
}

impl TrajectoryCollector {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::with_capacity(DEFAULT_MAX_BUFFER),
            max_buffer: DEFAULT_MAX_BUFFER,
            min_for_analysis: DEFAULT_MIN_ANALYSIS,
            last_flush: Instant::now(),
            flush_interval: DEFAULT_FLUSH_INTERVAL,
            policy_path: None,
            on_policy: None,
            total_recorded: 0,
            total_flushes: 0,
            last_policy: None,
        }
    }

    pub fn with_max_buffer(mut self, max: usize) -> Self {
        self.max_buffer = max;
        self.buffer = VecDeque::with_capacity(max);
        self
    }

    pub fn with_min_analysis(mut self, min: usize) -> Self {
        self.min_for_analysis = min;
        self
    }

    pub fn with_flush_interval(mut self, interval: Duration) -> Self {
        self.flush_interval = interval;
        self
    }

    pub fn with_policy_path(mut self, path: PathBuf) -> Self {
        self.policy_path = Some(path);
        self
    }

    pub fn with_policy_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + 'static,
    {
        self.on_policy = Some(Box::new(callback));
        self
    }

    pub fn record(&mut self, tool: &str, args: &serde_json::Value, error: bool, ts: u64) {
        let entry = TrajectoryEntry {
            tool: tool.to_string(),
            args: args.clone(),
            result: None,
            error,
            ts,
        };

        self.buffer.push_back(entry);
        self.total_recorded += 1;

        if self.buffer.len() >= self.max_buffer
            || self.last_flush.elapsed() >= self.flush_interval
        {
            self.flush();
        }
    }

    pub fn flush(&mut self) -> Option<String> {
        if self.buffer.len() < self.min_for_analysis {
            return None;
        }

        self.total_flushes += 1;
        self.last_flush = Instant::now();

        let entries: Vec<TrajectoryEntry> = self.buffer.drain(..).collect();
        let report = analyze_trajectory(&entries);
        let yaml = render_policy(&report);

        self.last_policy = Some(yaml.clone());

        if let Some(path) = &self.policy_path {
            if let Err(e) = std::fs::write(path, &yaml) {
                eprintln!(
                    "[TrajectoryCollector] Failed to write policy to {}: {e}",
                    path.display()
                );
            } else {
                eprintln!(
                    "[TrajectoryCollector] Auto-generated policy written to {} ({} calls, {} tools)",
                    path.display(),
                    report.total_calls,
                    report.tools.len(),
                );
            }
        }

        if let Some(ref callback) = self.on_policy {
            callback(&yaml);
        }

        Some(yaml)
    }

    pub fn buffered_count(&self) -> usize {
        self.buffer.len()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.last_flush = Instant::now();
    }
}

impl Default for TrajectoryCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_collector_starts_empty() {
        let collector = TrajectoryCollector::new();
        assert_eq!(collector.buffered_count(), 0);
        assert_eq!(collector.total_recorded, 0);
    }

    #[test]
    fn record_appends_to_buffer() {
        let mut collector = TrajectoryCollector::new();
        collector.record("read_file", &json!({"path": "x"}), false, 1);
        assert_eq!(collector.buffered_count(), 1);
        assert_eq!(collector.total_recorded, 1);
    }

    #[test]
    fn flush_requires_min_entries() {
        let mut collector = TrajectoryCollector::new()
            .with_min_analysis(5);
        for i in 0..3 {
            collector.record("tool", &json!({"i": i}), false, i);
        }
        assert_eq!(collector.total_flushes, 0);
        assert_eq!(collector.buffered_count(), 3);
    }

    #[test]
    fn auto_flush_on_buffer_full() {
        let mut collector = TrajectoryCollector::new()
            .with_max_buffer(5)
            .with_min_analysis(2);
        for i in 0..5 {
            collector.record("tool", &json!({"i": i}), false, i);
        }
        assert_eq!(collector.total_flushes, 1);
        assert_eq!(collector.buffered_count(), 0);
        assert!(collector.last_policy.is_some());
    }

    #[test]
    fn flush_generates_valid_yaml() {
        let mut collector = TrajectoryCollector::new()
            .with_min_analysis(2);
        collector.record("read_file", &json!({"path": "x"}), false, 1);
        collector.record("read_file", &json!({"path": "x"}), false, 2);
        collector.record("write_file", &json!({"path": "y"}), false, 3);

        let yaml = collector.flush();
        assert!(yaml.is_some());
        let yaml = yaml.unwrap();
        assert!(yaml.contains("read_file"));
        assert!(yaml.contains("write_file"));
        assert!(yaml.contains("max_repeats"));
    }

    #[test]
    fn callback_fires_on_flush() {
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let call_count_clone = call_count.clone();

        let mut collector = TrajectoryCollector::new()
            .with_min_analysis(2)
            .with_policy_callback(move |_policy| {
                call_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });

        collector.record("tool", &json!({}), false, 1);
        collector.record("tool", &json!({}), false, 2);
        collector.flush();

        assert_eq!(collector.total_flushes, 1);
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
