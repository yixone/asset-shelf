use actix_web::{HttpResponse, get, web};
use db::types::Pagination;
use models::types::AssetId;
use result::create_error;

use crate::{
    di::DataCtx,
    dto::v1::assets::SimilarAssetDtoV1,
    routes::{ApiResult, v1::assets::similar::searcher::SimilarSearcher},
};

#[get("/{id}/similar")]
async fn get_similar_asset(id: web::Path<AssetId>, ctx: web::Data<DataCtx>) -> ApiResult {
    if !ctx.config.instance.features.similar_search_enabled() {
        return Err(create_error!(FeatureDisabled {
            feature: "similar_search"
        }));
    }

    let reference = {
        let asset = ctx.db.assets.get_by_id(*id).await?;
        asset.features
    };

    let Some(color) = reference.accent_color else {
        return Ok(HttpResponse::NoContent().finish());
    };
    let (Some(width), Some(height)) = (reference.width, reference.height) else {
        return Ok(HttpResponse::NoContent().finish());
    };
    let aspect_ratio = width as f32 / height as f32;

    let candidates = ctx
        .db
        .assets
        .get_for_similar_search(color, aspect_ratio, Pagination::new(100, 0))
        .await?;

    let mut searcher = SimilarSearcher::new(reference);

    searcher.add_features(candidates);
    searcher.filter(40);
    searcher.sort_by_score();

    let similar = searcher.finalize();
    let assets = ctx.db.assets.get_from_similar(similar).await?;

    let res = assets
        .into_iter()
        .map(SimilarAssetDtoV1::from)
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(res))
}

mod searcher {
    use models::{
        assets::{
            AssetFeatures,
            similar::{SimilarAsset, SimilarScore},
        },
        types::Color,
    };

    pub struct SimilarSearcher {
        reference: AssetFeatures,
        features: Vec<SimilarAsset>,
    }

    const PHASH_WEIGHT: f32 = 0.60;
    const PHASH_MAX_DISTANCE: f32 = 35.0;

    const AHASH_WEIGHT: f32 = 0.65;
    const AHASH_MAX_DISTANCE: f32 = 30.0;

    const COLOR_WEIGHT: f32 = 0.3;

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
                if f.item.asset_id == self.reference.asset_id {
                    return false;
                }

                if !self.reference.is_similar_candidate_for(&f.item) {
                    return false;
                }

                let init_score = f.score.total_score();

                // Adds a pHash score
                let phash = calc_hash_score(
                    f.item.p_hash,
                    self.reference.p_hash,
                    PHASH_MAX_DISTANCE,
                    PHASH_WEIGHT,
                );

                // Adds a aHash score
                let ahash = calc_hash_score(
                    f.item.a_hash,
                    self.reference.a_hash,
                    AHASH_MAX_DISTANCE,
                    AHASH_WEIGHT,
                );

                // Adds a color score
                let color = calc_color_score(
                    f.item.accent_color,
                    self.reference.accent_color,
                    COLOR_WEIGHT,
                );

                f.score.phash += phash;
                f.score.ahash += ahash;
                f.score.color += color;

                let stage_score = f.score.total_score() - init_score;

                stage_score >= score_threshold
            });
        }

        pub fn sort_by_score(&mut self) {
            self.features
                .sort_by(|a, b| b.score.total_score().cmp(&a.score.total_score()));
        }

        pub fn finalize(self) -> Vec<SimilarAsset> {
            self.features
        }

        pub fn add_features(&mut self, feats: Vec<AssetFeatures>) {
            self.features
                .extend(feats.into_iter().map(|f| SimilarAsset {
                    item: f,
                    score: SimilarScore::new(),
                }));
        }
    }

    fn calc_hash_score(a: Option<i64>, b: Option<i64>, max_distance: f32, weight: f32) -> u32 {
        let (Some(a), Some(b)) = (a, b) else { return 0 };
        let dist = (a ^ b).count_ones();
        calc_weighed_score(dist as f32, max_distance, weight)
    }

    fn calc_color_score(a: Option<Color>, b: Option<Color>, weight: f32) -> u32 {
        let (Some(a), Some(b)) = (a, b) else { return 0 };

        let (a_r, a_g, a_b) = a.rgb();
        let (b_r, b_g, b_b) = b.rgb();

        let dist = ((a_r as u16).abs_diff(b_r as u16))
            + ((a_g as u16).abs_diff(b_g as u16))
            + ((a_b as u16).abs_diff(b_b as u16));

        calc_weighed_score(dist as f32, 765.0, weight)
    }

    fn calc_weighed_score(distance: f32, max_distance: f32, weight: f32) -> u32 {
        let similarity = 1.0 - (distance / max_distance);
        (similarity.max(0.0) * weight * 100.0).round() as u32
    }
}
