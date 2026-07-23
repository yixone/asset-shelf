//! tsid — a identifier based on a timestamp

#![allow(clippy::new_without_default)]

use std::sync::Mutex;

use chrono::Utc;

/// The starting epoch for generating timestamps
pub const DEFAULT_EPOCH: i64 = 1735678800000;

/// Runtime generator for FlakeId
///
/// ### Id parts
/// FlakeID consists of several parts:
/// `timestamp (44) | node_id (8) | idx (aka. sequence) (12)`
///
/// ### Collision and IDX
/// To prevent collisions within a single millisecond,
/// `idx` is used, enabling the generation of up
/// to 4096 IDS/ms for a given `node_id`
///
/// ### Usage
/// ```
/// use flake_id::{FlakeIdGenerator};
///
/// // Initializing the generator with a specific node_id
/// let generator = FlakeIdGenerator::new(1);
///
/// // Flake ID Generation
/// let id = generator.generate();
/// ```
pub struct FlakeIdGenerator {
    base_epoch: i64,
    node_id: u8,
    state: Mutex<GeneratorState>,
}

struct GeneratorState {
    idx: u16,
    last_time: i64,
}

/// Flake ID
///
/// ### About
/// Flake ID - an identifier inspired by Twitter's SnowflakeID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlakeId(pub i64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FlakeIdHex(String);

impl FlakeIdGenerator {
    pub fn new(node_id: u8) -> Self {
        FlakeIdGenerator::new_with_epoch(DEFAULT_EPOCH, node_id)
    }

    pub fn new_with_epoch(base_epoch: i64, node_id: u8) -> Self {
        let now = Utc::now().timestamp_millis();
        if now < base_epoch {
            panic!("System clock before generator base_epoch");
        }

        FlakeIdGenerator {
            base_epoch,
            node_id,
            state: Mutex::new(GeneratorState {
                idx: 0,
                last_time: 0,
            }),
        }
    }

    pub fn generate(&self) -> FlakeId {
        let (ts, idx) = {
            let mut state = self
                .state
                .lock()
                .expect("Failed to acquire the mutex for ID generation");

            let now = Utc::now().timestamp_millis();

            if state.last_time == now {
                state.idx += 1;
            } else {
                state.last_time = now;
                state.idx = 0
            }

            (now - self.base_epoch, state.idx)
        };
        FlakeId((ts << 20 | ((self.node_id as i64) << 12) | (idx as i64)) & (i64::MAX - 1))
    }

    pub fn generate_as<T>(&self) -> T
    where
        T: From<FlakeId>,
    {
        self.generate().into()
    }
}

impl From<FlakeId> for FlakeIdHex {
    fn from(id: FlakeId) -> Self {
        FlakeIdHex(format!("{:x}", id.0))
    }
}

impl std::fmt::Display for FlakeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for FlakeIdHex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
