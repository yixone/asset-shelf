#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(rename_all = "snake_case")
)]
#[derive(Debug)]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}
