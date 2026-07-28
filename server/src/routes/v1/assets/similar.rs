use actix_web::{HttpResponse, get, web};
use db::{
    database::DatabaseProvider,
    ops::{AssetFeaturesOps, AssetOps, MediaFilesOps},
    types::Pagination,
    utils::{bulk::CollectIds, join::JoinBuilder},
};
use models::types::AssetId;
use result::create_error;

use crate::{
    di::DataCtx,
    dto::v1::assets::AssetDtoV1,
    routes::{ApiResult, v1::assets::similar::searcher::SimilarSearcher},
};

#[get("/{id}/similar")]
async fn get_similar_asset(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    let reference = ctx
        .db
        .with_session(async |db| db.get_asset_features(&id).await)
        .await?
        .ok_or(create_error!(NotFound))?;

    let Some(color) = reference.accent_color else {
        return Ok(HttpResponse::NoContent().finish());
    };
    let Some(width) = reference.width else {
        return Ok(HttpResponse::NoContent().finish());
    };
    let Some(height) = reference.height else {
        return Ok(HttpResponse::NoContent().finish());
    };

    let aspect_ratio = width as f32 / height as f32;

    let candidates = ctx
        .db
        .with_session(async |db| {
            db.get_similarity_candidates(color, aspect_ratio, Pagination::new(100, 0))
                .await
        })
        .await?;

    let mut searcher = SimilarSearcher::new(reference);
    searcher.add_features(candidates);
    searcher.filter(40);
    searcher.sort_by_score();

    let similar = searcher.finalize();
    let mut conn = ctx.db.acquire().await?;
    let assets = conn.get_assets_bulk(&similar.ids()).await?;
    let media_files = conn.get_media_files_bulk(&assets.ids()).await?;
    drop(conn);

    let res = JoinBuilder::new(similar)
        .with(assets, |s| s)
        .with_group(media_files, |(_, a)| a)
        .transform(|((af, a), mf)| (a, af, mf))
        .build_as(AssetDtoV1::from);

    Ok(HttpResponse::Ok().json(res))
}

mod searcher {
    use models::entities::AssetFeatures;

    pub struct SimilarSearcher {
        reference: AssetFeatures,
        features: Vec<ScoredFeatures>,
    }

    pub struct ScoredFeatures {
        feature: AssetFeatures,
        score: u32,
    }

    const PHASH_WEIGHT: f32 = 0.50;
    const PHASH_DISTANCE: u32 = 25;

    const AHASH_WEIGHT: f32 = 0.75;
    const AHASH_DISTANCE: u32 = 15;

    const COLOR_WEIGHT: f32 = 0.5;
    const COLOR_DISTANCE: u32 = 16;

    impl SimilarSearcher {
        /// Creates a new [`SimilarSearcher`]
        pub fn new(reference: AssetFeatures) -> Self {
            SimilarSearcher {
                reference,
                features: Vec::new(),
            }
        }

        pub fn filter(&mut self, score_threshold: u32) {
            self.features.retain_mut(|f| {
                if f.feature.asset_id == self.reference.asset_id {
                    return false;
                }

                let init_score = f.score;

                // Adds a pHash score
                let phash = calc_weighed_score(
                    f.feature.p_hash,
                    self.reference.p_hash,
                    PHASH_DISTANCE,
                    PHASH_WEIGHT,
                );

                // Adds a aHash score
                let ahash = calc_weighed_score(
                    f.feature.a_hash,
                    self.reference.a_hash,
                    AHASH_DISTANCE,
                    AHASH_WEIGHT,
                );

                // Adds a color score
                let color = calc_weighed_score(
                    f.feature.accent_color.map(|c| c.0),
                    self.reference.accent_color.map(|c| c.0),
                    COLOR_DISTANCE,
                    COLOR_WEIGHT,
                );

                f.score += phash;
                f.score += ahash;
                f.score += color;

                // Adds a dimension score
                // TODO!

                let stage_score = f.score - init_score;
                tracing::info!(
                    phash,
                    ahash,
                    color,
                    stage_score,
                    score = f.score,
                    "Similar filtered"
                );
                stage_score >= score_threshold
            });
        }

        pub fn sort_by_score(&mut self) {
            self.features.sort_by_key(|f| f.score);
        }

        pub fn finalize(self) -> Vec<AssetFeatures> {
            self.features.into_iter().map(|f| f.feature).collect()
        }

        pub fn add_features(&mut self, feats: Vec<AssetFeatures>) {
            self.features
                .extend(feats.into_iter().map(|f| ScoredFeatures {
                    feature: f,
                    score: 0,
                }));
        }
    }

    fn calc_weighed_score<T>(a: Option<T>, b: Option<T>, distance: u32, weight: f32) -> u32
    where
        T: Into<i64>,
    {
        let (Some(a), Some(b)) = (a, b) else {
            return 0;
        };

        let dist = (a.into() ^ b.into()).count_ones();

        let similarity = 1.0 - (dist.min(distance) as f32 / distance as f32);

        (similarity * weight * 100.0).round() as u32
    }
}
