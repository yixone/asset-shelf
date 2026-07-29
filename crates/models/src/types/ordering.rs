#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum AssetsOrdering {
    /// Show newest assets first
    #[default]
    Newest,
    /// Show old assets first
    Oldest,
}

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize),
    serde(rename_all = "lowercase")
)]
pub enum CollectionAssetsOrdering {
    /// Show assets most recently added to the collection first
    #[default]
    Latest,
    /// Show the oldest assets added to the collection first
    Oldest,
}
