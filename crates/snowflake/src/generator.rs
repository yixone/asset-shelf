//! Contains a generator and its functions for generating `Snowflake Id`

use std::{
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::SnowflakeId;

/// The starting epoch for generating timestamps (`December 31, 2024`)
pub const DEFAULT_EPOCH: u64 = 1735678800000;

/// Bit shift for the `timestamp` part of the ID
const TIMESTAMP_SHIFT: usize = 22;

/// Bit shift for the `node id` part of the ID
const NODE_ID_SHIFT: usize = 14;

/// Sequence bit mask
const SEQUENCE_MASK: u16 = u16::MAX >> 2;

/// Generator for [`SnowflakeId`]
pub struct SnowflakeGenerator {
    /// The base epoch from which the generator begins generation
    base_epoch: SystemTime,
    /// Generator node identifier
    node_id: u8,
    /// Mutable state of the generator
    state: Mutex<GeneratorState>,
}

/// Internal state of the Snowflake generator
struct GeneratorState {
    seq: u16,
    last_time: i64,
}

impl SnowflakeGenerator {
    /// Creates a new [`SnowflakeGenerator`] with the [`DEFAULT_EPOCH`]
    pub fn new(node_id: u8) -> Self {
        Self::new_with_epoch(node_id, UNIX_EPOCH + Duration::from_millis(DEFAULT_EPOCH))
    }

    /// Creates a new [`SnowflakeGenerator`] with the specified base epoch
    pub fn new_with_epoch(node_id: u8, epoch: SystemTime) -> Self {
        let last_time = get_time_ms(epoch);
        SnowflakeGenerator {
            base_epoch: epoch,
            node_id,
            state: Mutex::new(GeneratorState { seq: 0, last_time }),
        }
    }

    /// Generates the next [`SnowflakeId`] using this generator and the time the method was called
    pub fn get_id(&self) -> SnowflakeId {
        // Acquires a lock on the generator state
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());

        // Gets the current time (minus base_epoch)
        let mut now = get_time_ms(self.base_epoch);

        // If the last generation occurred within the same millisecond, continues the sequence counter
        if now == state.last_time {
            // Increments the sequence and applies a mask
            state.seq = (state.seq + 1) & SEQUENCE_MASK;

            // If the sequence hits the limit for a given millisecond, waits for the next millisecond
            if state.seq == 0 {
                now = til_next_ms(self.base_epoch, state.last_time);
                state.last_time = now;
            }
        } else {
            // If the last generation did not occur in the current millisecond, resets the sequence counter
            state.last_time = now;
            state.seq = 0;
        }

        // Casts all ID segments to i64
        let time = state.last_time;
        let node = self.node_id as i64;
        let sequence = state.seq as i64;

        // Assembles the ID from segments and excludes the sign bit using the `i64::MAX` mask
        let int_id = (time << TIMESTAMP_SHIFT) | (node << NODE_ID_SHIFT) | sequence;
        SnowflakeId(int_id & i64::MAX)
    }

    /// Generates the next [`SnowflakeId`] and returns it as the generic type
    pub fn get_id_as<T>(&self) -> T
    where
        T: From<SnowflakeId>,
    {
        self.get_id().into()
    }
}

/// Get the time elapsed since `base_epoch` in milliseconds
fn get_time_ms(epoch: SystemTime) -> i64 {
    SystemTime::now()
        .duration_since(epoch)
        .expect("System clock before generator base_epoch")
        .as_millis() as i64
}

/// Waiting for the next millisecond
fn til_next_ms(epoch: SystemTime, last_time: i64) -> i64 {
    loop {
        let ts = get_time_ms(epoch);
        if ts > last_time {
            return ts;
        }
        std::hint::spin_loop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_different_snowflake_id() {
        let generator = SnowflakeGenerator::new(0);

        let id = generator.get_id();
        let id2 = generator.get_id();

        assert_ne!(id, id2);
    }
}
