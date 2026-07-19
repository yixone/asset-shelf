#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetsOrdering {
    #[default]
    Newest,
    Oldest,
}
