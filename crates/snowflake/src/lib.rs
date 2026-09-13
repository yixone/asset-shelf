//! Crate for working with [`SnowflakeId`]
//!
//! ### Generation:
//! IDs can be generated directly via the generator
//! ```
//! use snowflake::SnowflakeGenerator;
//!
//! let generator = SnowflakeGenerator::new(0);
//! let id = generator.get_id();
//! ```

mod generator;

mod id_int;
mod id_str;

pub use generator::SnowflakeGenerator;

pub use id_int::SnowflakeId;
pub use id_str::SnowflakeIdStr;
