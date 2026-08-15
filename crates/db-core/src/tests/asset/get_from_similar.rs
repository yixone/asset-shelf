#![allow(clippy::disallowed_names)]

use models::assets::similar::{SimilarAsset, SimilarScore};

use super::*;

pub async fn from_similar_searcher_results<R: AssetRepository>(repo: R) -> Result<()> {
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
