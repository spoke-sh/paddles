use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::time::Duration;
use std::{fs, io};

const WINDOW_CAP: usize = 50;
const MIN_SAMPLES: usize = 5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pace {
    Fast,
    Normal,
    Slow,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StepTimingReservoir {
    windows: BTreeMap<String, VecDeque<u64>>,
}

impl StepTimingReservoir {
    pub fn record(&mut self, key: &str, delta: Duration) {
        let window = self.windows.entry(key.to_string()).or_default();
        if window.len() >= WINDOW_CAP {
            window.pop_front();
        }
        window.push_back(delta.as_millis() as u64);
    }

    pub fn percentile(&self, key: &str, p: u8) -> Option<Duration> {
        let window = self.windows.get(key)?;
        if window.len() < MIN_SAMPLES {
            return None;
        }
        let mut sorted: Vec<u64> = window.iter().copied().collect();
        sorted.sort_unstable();
        let rank = ((p as f64 / 100.0) * sorted.len() as f64).ceil().max(1.0) as usize;
        let index = rank.min(sorted.len()) - 1;
        Some(Duration::from_millis(sorted[index]))
    }

    pub fn classify(&self, key: &str, delta: Duration) -> Pace {
        let Some(p50) = self.percentile(key, 50) else {
            return Pace::Normal;
        };
        if delta < p50 {
            return Pace::Fast;
        }
        let Some(p85) = self.percentile(key, 85) else {
            return Pace::Normal;
        };
        if delta > p85 {
            Pace::Slow
        } else {
            Pace::Normal
        }
    }

    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    }

    pub fn flush(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let data = serde_json::to_string(self).map_err(io::Error::other)?;
        fs::write(path, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn ms(millis: u64) -> Duration {
        Duration::from_millis(millis)
    }

    #[test]
    fn record_stores_deltas_up_to_window_cap() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 0..50 {
            reservoir.record("tool_called", ms(i * 10));
        }
        assert_eq!(reservoir.windows["tool_called"].len(), 50);

        reservoir.record("tool_called", ms(999));
        assert_eq!(reservoir.windows["tool_called"].len(), 50);
        assert_eq!(*reservoir.windows["tool_called"].back().unwrap(), 999);
    }

    #[test]
    fn oldest_entries_evicted_when_window_full() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 0..50 {
            reservoir.record("key", ms(i));
        }
        assert_eq!(*reservoir.windows["key"].front().unwrap(), 0);

        reservoir.record("key", ms(9999));
        assert_eq!(*reservoir.windows["key"].front().unwrap(), 1);
        assert_eq!(*reservoir.windows["key"].back().unwrap(), 9999);
    }

    #[test]
    fn percentile_returns_none_with_insufficient_samples() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 0..4 {
            reservoir.record("sparse", ms(i * 100));
        }
        assert!(reservoir.percentile("sparse", 50).is_none());
        assert!(reservoir.percentile("nonexistent", 50).is_none());
    }

    #[test]
    fn percentile_computes_correct_p50_and_p85() {
        let mut reservoir = StepTimingReservoir::default();
        // Insert 20 values: 100, 200, 300, ..., 2000
        for i in 1..=20 {
            reservoir.record("key", ms(i * 100));
        }
        let p50 = reservoir.percentile("key", 50).unwrap();
        assert_eq!(p50, ms(1000));

        let p85 = reservoir.percentile("key", 85).unwrap();
        assert_eq!(p85, ms(1700));
    }

    #[test]
    fn classify_returns_normal_with_insufficient_history() {
        let mut reservoir = StepTimingReservoir::default();
        reservoir.record("sparse", ms(100));
        assert_eq!(reservoir.classify("sparse", ms(50)), Pace::Normal);
    }

    #[test]
    fn classify_returns_fast_below_p50() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 1..=20 {
            reservoir.record("key", ms(i * 100));
        }
        // p50 = 1000ms, so 500ms should be Fast
        assert_eq!(reservoir.classify("key", ms(500)), Pace::Fast);
    }

    #[test]
    fn classify_returns_normal_between_p50_and_p85() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 1..=20 {
            reservoir.record("key", ms(i * 100));
        }
        // p50 = 1000ms, p85 = 1700ms, so 1200ms should be Normal
        assert_eq!(reservoir.classify("key", ms(1200)), Pace::Normal);
    }

    #[test]
    fn classify_returns_slow_above_p85() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 1..=20 {
            reservoir.record("key", ms(i * 100));
        }
        // p85 = 1700ms, so 2500ms should be Slow
        assert_eq!(reservoir.classify("key", ms(2500)), Pace::Slow);
    }

    #[test]
    fn round_trips_through_json_serialization() {
        let mut reservoir = StepTimingReservoir::default();
        for i in 1..=10 {
            reservoir.record("alpha", ms(i * 50));
            reservoir.record("beta", ms(i * 200));
        }
        let json = serde_json::to_string(&reservoir).unwrap();
        let restored: StepTimingReservoir = serde_json::from_str(&json).unwrap();
        assert_eq!(reservoir.windows, restored.windows);
    }

    #[test]
    fn load_returns_empty_for_missing_file() {
        let reservoir = StepTimingReservoir::load(Path::new("/tmp/paddles_test_nonexistent.json"));
        assert!(reservoir.windows.is_empty());
    }

    #[test]
    fn load_returns_empty_for_corrupt_file() {
        let path = std::env::temp_dir().join("paddles_test_corrupt.json");
        fs::write(&path, "not valid json{{{").unwrap();
        let reservoir = StepTimingReservoir::load(&path);
        assert!(reservoir.windows.is_empty());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn flush_creates_cache_directory() {
        let dir = std::env::temp_dir().join("paddles_test_cache_dir_create");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("step_timing.json");

        let mut reservoir = StepTimingReservoir::default();
        reservoir.record("test", ms(100));
        reservoir.flush(&path).unwrap();

        let restored = StepTimingReservoir::load(&path);
        assert_eq!(restored.windows["test"].len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }
}
