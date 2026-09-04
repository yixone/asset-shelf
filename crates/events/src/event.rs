use std::{any::Any, fmt::Debug};

pub trait DynamicEvent: Any + Send + Sync + Debug {}
