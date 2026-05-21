//! Hybrid Logical Clock.
//!
//! Each mutation in Hoverdo is tagged with an HLC timestamp so that future
//! multi-device sync can resolve conflicts deterministically (last-writer-wins
//! ordered by HLC, with the node id breaking ties). Even though MVP is
//! offline-only, baking the clock in from day one means no schema migration
//! when sync arrives.
//!
//! Serialization format (38 chars, fixed width, lexicographically sortable):
//!
//! ```text
//! <ts_ms:16hex>-<counter:4hex>-<node_id:16hex>
//! ```
//!
//! Reference: Kulkarni et al., "Logical Physical Clocks and Consistent
//! Snapshots in Globally Distributed Databases", 2014.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{HoverdoError, Result};

/// A single HLC value. Cheap to clone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hlc {
    pub ts_ms: u64,
    pub counter: u16,
    pub node_id: [u8; 8],
}

impl Hlc {
    pub fn new(ts_ms: u64, counter: u16, node_id: [u8; 8]) -> Self {
        Self {
            ts_ms,
            counter,
            node_id,
        }
    }
}

impl std::fmt::Display for Hlc {
    /// 38-char lex-sortable hex serialization.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}-{:04x}-", self.ts_ms, self.counter)?;
        for b in &self.node_id {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

impl std::str::FromStr for Hlc {
    type Err = HoverdoError;

    fn from_str(s: &str) -> Result<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 3 || parts[0].len() != 16 || parts[1].len() != 4 || parts[2].len() != 16
        {
            return Err(HoverdoError::InvalidHlc(s.to_string()));
        }
        let ts_ms = u64::from_str_radix(parts[0], 16)
            .map_err(|_| HoverdoError::InvalidHlc(s.to_string()))?;
        let counter = u16::from_str_radix(parts[1], 16)
            .map_err(|_| HoverdoError::InvalidHlc(s.to_string()))?;
        let mut node_id = [0u8; 8];
        for (i, byte) in node_id.iter_mut().enumerate() {
            let hi = parts[2]
                .as_bytes()
                .get(i * 2)
                .and_then(|b| (*b as char).to_digit(16))
                .ok_or_else(|| HoverdoError::InvalidHlc(s.to_string()))?;
            let lo = parts[2]
                .as_bytes()
                .get(i * 2 + 1)
                .and_then(|b| (*b as char).to_digit(16))
                .ok_or_else(|| HoverdoError::InvalidHlc(s.to_string()))?;
            *byte = ((hi << 4) | lo) as u8;
        }
        Ok(Self {
            ts_ms,
            counter,
            node_id,
        })
    }
}

/// Stateful HLC generator. Hold one per `Db` instance.
pub struct HlcClock {
    node_id: [u8; 8],
    state: Mutex<HlcState>,
}

#[derive(Debug, Default, Clone, Copy)]
struct HlcState {
    ts_ms: u64,
    counter: u16,
}

impl HlcClock {
    pub fn new(node_id: [u8; 8]) -> Self {
        Self {
            node_id,
            state: Mutex::new(HlcState::default()),
        }
    }

    pub fn node_id(&self) -> [u8; 8] {
        self.node_id
    }

    /// Issue the next local HLC, monotonically greater than any previously
    /// issued or observed HLC.
    pub fn now(&self) -> Hlc {
        let wall = wall_ms();
        let mut state = self.state.lock().expect("HLC mutex poisoned");
        if wall > state.ts_ms {
            state.ts_ms = wall;
            state.counter = 0;
        } else {
            state.counter = state.counter.saturating_add(1);
        }
        Hlc::new(state.ts_ms, state.counter, self.node_id)
    }

    /// Incorporate a remote HLC (e.g. received during sync) and return the
    /// next local HLC strictly greater than both the previous local and the
    /// remote value.
    pub fn observe(&self, remote: &Hlc) -> Hlc {
        let wall = wall_ms();
        let mut state = self.state.lock().expect("HLC mutex poisoned");
        let max_ts = state.ts_ms.max(remote.ts_ms).max(wall);
        if max_ts == state.ts_ms && max_ts == remote.ts_ms {
            state.counter = state.counter.max(remote.counter).saturating_add(1);
        } else if max_ts == state.ts_ms {
            state.counter = state.counter.saturating_add(1);
        } else if max_ts == remote.ts_ms {
            state.counter = remote.counter.saturating_add(1);
        } else {
            state.counter = 0;
        }
        state.ts_ms = max_ts;
        Hlc::new(state.ts_ms, state.counter, self.node_id)
    }
}

fn wall_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn node() -> [u8; 8] {
        [1, 2, 3, 4, 5, 6, 7, 8]
    }

    #[test]
    fn string_roundtrip() {
        let h = Hlc::new(0x0123456789abcdef, 0xbeef, node());
        let s = h.to_string();
        assert_eq!(s.len(), 38);
        let parsed = Hlc::from_str(&s).unwrap();
        assert_eq!(parsed, h);
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert!(Hlc::from_str("not an hlc").is_err());
        assert!(Hlc::from_str("000000000000000z-0000-0000000000000000").is_err());
        assert!(Hlc::from_str("0000000000000000-0000").is_err()); // missing field
    }

    #[test]
    fn now_is_strictly_monotonic() {
        let clock = HlcClock::new(node());
        let mut prev = clock.now();
        for _ in 0..1_000 {
            let next = clock.now();
            assert!(next > prev, "HLC went backwards: {prev:?} -> {next:?}");
            prev = next;
        }
    }

    #[test]
    fn observe_advances_past_remote() {
        let clock = HlcClock::new(node());
        let remote = Hlc::new(wall_ms() + 60_000, 5, [9; 8]);
        let next = clock.observe(&remote);
        assert!(
            next > remote,
            "observe should produce HLC > remote; got {next:?} vs {remote:?}"
        );
        let after = clock.now();
        assert!(after > next);
    }

    #[test]
    fn lex_sort_matches_logical_order() {
        let a = Hlc::new(100, 0, node()).to_string();
        let b = Hlc::new(100, 1, node()).to_string();
        let c = Hlc::new(101, 0, node()).to_string();
        assert!(a < b);
        assert!(b < c);
    }
}
