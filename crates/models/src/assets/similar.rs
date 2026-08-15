use crate::assets::AssetFeatures;

/// A model containing data for ranking and searching
/// for similar assets within the service
#[derive(Debug, Clone, PartialEq)]
pub struct SimilarAsset {
    pub item: AssetFeatures,
    pub score: SimilarScore,
}

/// Scores for attributes [`SimilarAsset`]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct SimilarScore {
    pub color: u32,
    pub ahash: u32,
    pub phash: u32,
}

impl SimilarScore {
    /// Creates a new [`SimilarScore`]
    pub fn new() -> Self {
        Self {
            color: 0,
            ahash: 0,
            phash: 0,
        }
    }

    /// Returns the total score of this [`SimilarScore`]
    pub fn total_score(&self) -> u32 {
        self.color + self.ahash + self.phash
    }
}
