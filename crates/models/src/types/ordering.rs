#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum AssetsOrdering {
    #[default]
    Newest,
    Oldest,
}
