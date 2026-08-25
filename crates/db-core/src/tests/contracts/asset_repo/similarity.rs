#![allow(clippy::disallowed_names)]

use chrono::Utc;
use models::{
    assets::similar::{SimilarAsset, SimilarScore},
    types::Color,
};

use crate::types::Pagination;

use super::*;

/// Checks that the given [`AssetRepository`] returns suitable candidates
/// for searching similar assets and does not receive unnecessary ones
pub async fn return_candidates_for_similar_search<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let r#ref = {
        let (_, _, mut af) = prepare_asset(&flake, "ref");

        af.a_hash = Some(i64::MAX);
        af.p_hash = Some(i64::MAX);
        af.accent_color = Some(Color::from(i32::MAX));

        af.width = Some(1000);
        af.height = Some(1);

        af
    };

    let similar = {
        let (m, a, mut af) = prepare_asset(&flake, "similar");

        af.a_hash = Some(i64::MAX >> 10);
        af.p_hash = Some(i64::MAX >> 8);
        af.accent_color = Some(Color::from(i32::MAX >> 4));

        af.width = Some(900);
        af.height = Some(50);

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(r#ref.is_similar_candidate_for(&af));

        af
    };

    let similar_deleted = {
        let (m, mut a, mut af) = prepare_asset(&flake, "similar deleted");

        af.a_hash = Some(i64::MAX >> 10);
        af.p_hash = Some(i64::MAX >> 8);
        af.accent_color = Some(Color::from(i32::MAX >> 4));

        af.width = Some(900);
        af.height = Some(50);

        a.deleted_at = Some(Utc::now());

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(r#ref.is_similar_candidate_for(&af));

        af
    };

    let not_similar = {
        let (m, a, mut af) = prepare_asset(&flake, "not similar");

        af.a_hash = Some(0);
        af.p_hash = Some(0);
        af.accent_color = Some(Color::from(0));

        af.width = Some(1);
        af.height = Some(1000);

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(!r#ref.is_similar_candidate_for(&af));

        af
    };

    let similar_list = repo
        .get_for_similar_search(
            r#ref.accent_color.unwrap(),
            r#ref.width.unwrap() as f32 / r#ref.height.unwrap() as f32,
            Pagination::new(50, 0),
        )
        .await?;

    assert_eq!(similar_list.len(), 1);

    assert!(similar_list.contains(&similar));
    assert!(!similar_list.contains(&similar_deleted));

    assert!(
        similar_list
            .iter()
            .all(|f| f.is_similar_candidate_for(&r#ref))
    );

    assert!(!similar_list.contains(&not_similar));

    Ok(())
}

/// Checks that this [`AssetRepository`] correctly retrieves a list of assets
/// in the correct order based on similarity search results
pub async fn list_from_similar_search_results<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);

    let foo = {
        let (m, a, mut af) = prepare_asset(&flake, "foo");

        af.a_hash = Some(i64::MAX);
        af.p_hash = Some(i64::MAX);

        af.width = Some(1000);
        af.height = Some(1);

        insert_full_asset((&m, &a, &af), &repo).await?;

        (a, af)
    };

    let bar = {
        let (m, a, mut af) = prepare_asset(&flake, "bar");

        af.a_hash = Some(i64::MAX >> 10);
        af.p_hash = Some(i64::MAX >> 8);

        af.width = Some(900);
        af.height = Some(50);

        insert_full_asset((&m, &a, &af), &repo).await?;

        (a, af)
    };

    let bazz = {
        let (m, a, mut af) = prepare_asset(&flake, "bazz");

        af.a_hash = Some(0);
        af.p_hash = Some(0);

        af.width = Some(1);
        af.height = Some(1000);

        insert_full_asset((&m, &a, &af), &repo).await?;

        (a, af)
    };

    let mut a = SimilarAsset {
        item: bar.1,
        score: SimilarScore::new(),
    };
    a.score.color = 50;

    let mut b = SimilarAsset {
        item: foo.1,
        score: SimilarScore::new(),
    };
    b.score.ahash = 42;

    let c = SimilarAsset {
        item: bazz.1,
        score: SimilarScore::new(),
    };

    let input = vec![a.clone(), b.clone(), c.clone()];

    let out = repo.get_from_similar(input).await?;

    assert_eq!(out.len(), 3);

    assert_eq!(out[0].asset.inner, bar.0);
    assert_eq!(out[1].asset.inner, foo.0);
    assert_eq!(out[2].asset.inner, bazz.0);

    assert_eq!(out[0].score, a);
    assert_eq!(out[1].score, b);
    assert_eq!(out[2].score, c);

    Ok(())
}
