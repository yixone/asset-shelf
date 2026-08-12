use std::sync::Arc;

pub mod asset;
pub mod collection;
pub mod media;

pub struct RepositoryContext {
    pub assets: Arc<dyn asset::AssetRepository>,
    pub collections: Arc<dyn collection::CollectionRepository>,
    pub media: Arc<dyn media::MediaRepository>,
}
