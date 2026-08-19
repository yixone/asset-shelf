use db::types::patch::AssetFeaturesPatch;
use models::types::Color;

pub mod image;
pub mod video;

pub struct ExtractedFeatures {
    pub a_hash: i64,
    pub p_hash: i64,
    pub color: Color,
    pub width: u32,
    pub height: u32,
}

impl From<ExtractedFeatures> for AssetFeaturesPatch {
    fn from(f: ExtractedFeatures) -> Self {
        AssetFeaturesPatch::new()
            .a_hash(Some(f.a_hash))
            .p_hash(Some(f.p_hash))
            .height(Some(f.height))
            .width(Some(f.width))
            .accent_color(Some(f.color))
    }
}
