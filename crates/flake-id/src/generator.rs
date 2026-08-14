use std::sync::Mutex;

use chrono::Utc;

use crate::FlakeId;

/// The starting epoch for generating timestamps
pub const DEFAULT_EPOCH: i64 = 1735678800000;

const TIMESTAMP_SHIFT: usize = 22;
const NODE_ID_SHIFT: usize = 14;

const SEQUENCE_MASK: u16 = u16::MAX >> 2;

/// Runtime generator for FlakeId
///
/// ### Id parts
/// FlakeID consists of several parts:
/// `timestamp (42) | node_id (8) | sequence (14)`
///
/// ### Collision and IDX
/// To prevent collisions within a single millisecond,
/// `idx` is used, enabling the generation of up
/// to 16383 IDS/ms for a given `node_id`
///
/// ### Usage
/// ```
/// use flake_id::{FlakeIdGenerator};
///
/// // Initializing the generator with a specific node_id
/// let generator = FlakeIdGenerator::new(1);
///
/// // Flake ID Generation
/// let id = generator.get_id();
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

    pub fn get_id(&self) -> FlakeId {
        let mut state = self
            .state
            .lock()
            .expect("Failed to acquire the mutex for ID generation");

        let mut now = get_time_now(self.base_epoch);

        if now == state.last_time {
            state.idx = (state.idx + 1) & SEQUENCE_MASK;
            if state.idx == 0 {
                now = til_next_ms(self.base_epoch, state.last_time);
                state.last_time = now;
            }
        } else {
            state.last_time = now;
            state.idx = 0
        }

        let time = state.last_time;
        let node = self.node_id as i64;
        let sequence = state.idx as i64;

        // [ TIMESTAMP (41) ] | [ NODE_ID (8) ] | [ SEQUENCE (14) ]
        FlakeId((time << TIMESTAMP_SHIFT | (node << NODE_ID_SHIFT) | sequence) & i64::MAX)
    }

    pub fn get_id_as<T>(&self) -> T
    where
        T: From<FlakeId>,
    {
        self.get_id().into()
    }
}

fn get_time_now(base_epoch: i64) -> i64 {
    let now = Utc::now().timestamp_millis();
    now - base_epoch
}

fn til_next_ms(base_epoch: i64, last_time: i64) -> i64 {
    loop {
        let ts = get_time_now(base_epoch);
        if ts > last_time {
            return ts;
        }
        std::hint::spin_loop();
    }
}
