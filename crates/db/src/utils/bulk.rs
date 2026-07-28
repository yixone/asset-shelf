use models::{
    entities::{Asset, AssetFeatures, Collection, Media, MediaFile},
    types::{AssetId, CollectionId, MediaFileId, MediaId},
};

pub trait CollectIds<T> {
    fn ids(&self) -> Vec<T>;
}

impl CollectIds<AssetId> for Vec<Asset> {
    fn ids(&self) -> Vec<AssetId> {
        self.iter().map(|a| a.id).collect()
    }
}
impl CollectIds<MediaId> for Vec<Asset> {
    fn ids(&self) -> Vec<MediaId> {
        self.iter().map(|a| a.media_id.clone()).collect()
    }
}

impl CollectIds<MediaId> for Vec<Media> {
    fn ids(&self) -> Vec<MediaId> {
        self.iter().map(|m| m.id.clone()).collect()
    }
}

impl CollectIds<MediaId> for Vec<MediaFile> {
    fn ids(&self) -> Vec<MediaId> {
        self.iter().map(|m| m.media_id.clone()).collect()
    }
}
impl CollectIds<MediaFileId> for Vec<MediaFile> {
    fn ids(&self) -> Vec<MediaFileId> {
        self.iter().map(|m| m.id.clone()).collect()
    }
}

impl CollectIds<AssetId> for Vec<AssetFeatures> {
    fn ids(&self) -> Vec<AssetId> {
        self.iter().map(|a| a.asset_id).collect()
    }
}

impl CollectIds<CollectionId> for Vec<Collection> {
    fn ids(&self) -> Vec<CollectionId> {
        self.iter().map(|c| c.id).collect()
    }
}
