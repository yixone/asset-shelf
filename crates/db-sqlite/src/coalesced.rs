use std::collections::HashMap;

use db_api::{DatabaseError, types::AssetFull};
use models::{
    entities::{Asset, FileGroup},
    id::FileGroupId,
};
use sqlx::SqliteConnection;

use crate::{
    operations,
    schema::{DbAsset, DbAssetFeatures, DbAssetMeta, DbFile, DbFileGroup},
};

fn unit_files_domain(
    groups: Vec<DbFileGroup>,
    files: Vec<DbFile>,
) -> Result<Vec<FileGroup>, DatabaseError> {
    if groups.is_empty() {
        return Ok(vec![]);
    }

    let mut files_idx: HashMap<FileGroupId, Vec<DbFile>> = HashMap::with_capacity(groups.len());
    for f in files {
        if let Some(fm) = files_idx.get_mut(&f.group_id) {
            fm.push(f);
        } else {
            files_idx.insert(f.group_id.clone(), vec![f]);
        }
    }

    groups
        .into_iter()
        .filter_map(|g| {
            let files = files_idx.remove(&g.id)?;
            Some((g, files))
        })
        .map(DbFileGroup::into_domain)
        .collect::<Result<Vec<_>, DatabaseError>>()
}

fn unit_assets_domain(
    assets: Vec<DbAsset>,
    features: Vec<DbAssetFeatures>,
    meta: Vec<DbAssetMeta>,
) -> Result<Vec<Asset>, DatabaseError> {
    let mut features_idx = features
        .into_iter()
        .map(|f| (f.id, f))
        .collect::<HashMap<_, _>>();
    let mut meta_idx = meta
        .into_iter()
        .map(|m| (m.id, m))
        .collect::<HashMap<_, _>>();

    assets
        .into_iter()
        .filter_map(|a| {
            let features = features_idx.remove(&a.id)?;
            let meta = meta_idx.remove(&a.id)?;

            Some((a, features, meta))
        })
        .map(DbAsset::into_domain)
        .collect::<Result<Vec<_>, _>>()
}

fn unit_assets_full(assets: Vec<Asset>, files: Vec<FileGroup>) -> Vec<AssetFull> {
    let mut files_idx = files
        .into_iter()
        .map(|f| (f.id().clone(), f))
        .collect::<HashMap<_, _>>();

    assets
        .into_iter()
        .filter_map(|a| {
            let files = files_idx.remove(a.file_group_id())?;
            Some((a, files))
        })
        .collect::<Vec<_>>()
}

pub async fn hydrate_file_groups(
    groups: Vec<DbFileGroup>,
    conn: &mut SqliteConnection,
) -> Result<Vec<FileGroup>, DatabaseError> {
    let ids = groups.iter().map(|g| g.id.clone()).collect::<Vec<_>>();
    let files = operations::load_files_by_groups(&ids, conn).await?;
    unit_files_domain(groups, files)
}

pub async fn hydrate_file_group(
    group: DbFileGroup,
    conn: &mut SqliteConnection,
) -> Result<FileGroup, DatabaseError> {
    let files = operations::load_files_by_group(&group.id, conn).await?;
    DbFileGroup::into_domain((group, files))
}

pub async fn hydrate_assets(
    assets: Vec<DbAsset>,
    conn: &mut SqliteConnection,
) -> Result<Vec<Asset>, DatabaseError> {
    let ids = assets.iter().map(|a| a.id).collect::<Vec<_>>();
    let features = operations::load_asset_features_many(&ids, conn).await?;
    let meta = operations::load_asset_meta_many(&ids, conn).await?;
    unit_assets_domain(assets, features, meta)
}

pub async fn hydrate_asset(
    asset: DbAsset,
    conn: &mut SqliteConnection,
) -> Result<Asset, DatabaseError> {
    let features = operations::load_asset_features(asset.id, conn)
        .await?
        .ok_or(DatabaseError::NotFound)?;
    let meta = operations::load_asset_meta(asset.id, conn)
        .await?
        .ok_or(DatabaseError::NotFound)?;
    DbAsset::into_domain((asset, features, meta))
}

pub async fn hydrate_assets_full(
    assets: Vec<DbAsset>,
    conn: &mut SqliteConnection,
) -> Result<Vec<AssetFull>, DatabaseError> {
    let assets = hydrate_assets(assets, conn).await?;

    let file_groups_ids = assets
        .iter()
        .map(|a| a.file_group_id().clone())
        .collect::<Vec<_>>();

    let file_groups = operations::load_file_groups(&file_groups_ids, conn).await?;
    let files = operations::load_files_by_groups(&file_groups_ids, conn).await?;

    let files = unit_files_domain(file_groups, files)?;

    Ok(unit_assets_full(assets, files))
}

pub async fn hydrate_asset_full(
    asset: DbAsset,
    conn: &mut SqliteConnection,
) -> Result<AssetFull, DatabaseError> {
    let asset = hydrate_asset(asset, conn).await?;

    let file_group = operations::load_file_group(asset.file_group_id(), conn)
        .await?
        .ok_or(DatabaseError::NotFound)?;
    let files = operations::load_files_by_group(asset.file_group_id(), conn).await?;
    let files = DbFileGroup::into_domain((file_group, files))?;

    Ok((asset, files))
}
