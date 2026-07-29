use crate::{
    entities::{Asset, AssetFeatures, Collection, CollectionAsset, Media, MediaFile},
    types::{AssetId, CollectionId, MediaFileId, MediaId},
};

pub trait BulkIds<T> {
    fn ids(&self) -> Vec<T>;
}

macro_rules! bulk_ids {
    ( $a: ty => $b: ty: $id:expr ) => {
        impl BulkIds<$b> for Vec<$a> {
            fn ids(&self) -> Vec<$b> {
                self.iter().map(|a| ($id)(a)).collect()
            }
        }
    };
}

bulk_ids!(Asset => AssetId: |a: &Asset| a.id);
bulk_ids!(Asset => MediaId: |a: &Asset| a.media_id.clone());

bulk_ids!(AssetFeatures => AssetId: |a: &AssetFeatures| a.asset_id);

bulk_ids!(Media => MediaId: |m: &Media| m.id.clone());

bulk_ids!(MediaFile => MediaFileId: |m: &MediaFile| m.id.clone());
bulk_ids!(MediaFile => MediaId: |m: &MediaFile| m.media_id.clone());

bulk_ids!(Collection => CollectionId: |c: &Collection| c.id);

bulk_ids!(CollectionAsset => AssetId: |c: &CollectionAsset| c.asset_id);
bulk_ids!(CollectionAsset => CollectionId: |c: &CollectionAsset| c.collection_id);
