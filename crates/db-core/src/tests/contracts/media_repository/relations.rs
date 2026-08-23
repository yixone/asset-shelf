use flake_id::FlakeIdGenerator;
use result::Result;

use crate::{
    repos::{asset::AssetRepository, media::MediaRepository},
    tests::contracts::{
        asset_repository::{insert_full_asset, prepare_asset},
        media_repository::prepare_media,
    },
};

/// Testing the return of an orphaned media group
pub async fn get_orphans<R: MediaRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let media = prepare_media(&flake);

    repo.insert(&media).await?;

    let orhans = repo.get_orphans(50).await?;

    assert_eq!(orhans.len(), 1);
    assert_eq!(orhans[0].inner.id, media.id);

    Ok(())
}

/// Tests that if there are no orphaned files, an empty array will be returned.
pub async fn get_empty_for_no_orphans<AR: AssetRepository, MR: MediaRepository>(
    asset_repo: AR,
    media_repo: MR,
) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let (media, asset, af) = prepare_asset(&flake, "Test");
    insert_full_asset((&media, &asset, &af), &asset_repo).await?;

    let orhans = media_repo.get_orphans(50).await?;

    assert!(orhans.is_empty());

    Ok(())
}
