use chrono::Utc;
use models::{assets::AssetState, types::Color};

use super::*;

/// Ensures that only suitable [`Asset`] are returned for processing
///
/// Performs a check for all asset states:
/// - Newly created asset ([`AssetState::Pending`])
/// - Asset taken for processing ([`AssetState::Processing`] + Recently updated)
/// - Processing is broken without changing state ([`AssetState::Processing`] + Modified long ago)
/// - The asset is processed and ready ([`AssetState::Ready`] + All fields is [`Some`])
/// - The asset was processed earlier, but now it does not have all the fields ([`AssetState::Ready`] + Not all fields are [`Some`])
/// - Processing failed recently ([`AssetState::Failed`] + Recently updated)
/// - Processing ended with an error long ago ([`AssetState::Failed`] + Modified long ago)
pub async fn get_for_processing<R: AssetRepository>(repo: R) -> Result<()> {
    let flake = FlakeIdGenerator::new(0);
    let now = Utc::now();

    // Creates assets that implement all possible states
    let pending = {
        let (m, a, af) = prepare_asset(&flake, "pending");
        insert_full_asset((&m, &a, &af), &repo).await?;
        assert!(a.need_processing(&af, Utc::now()));
        a
    };

    let processing_fresh = {
        let (m, mut a, af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Processing;

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(!a.need_processing(&af, now));

        a
    };

    let processing_stale = {
        let (m, mut a, af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Processing;
        a.updated_at = now - Asset::TIME_BEFORE_REPROCESSING;

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(a.need_processing(&af, now));

        a
    };

    let ready_complete = {
        let (m, mut a, mut af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Ready;

        af.a_hash = Some(0);
        af.p_hash = Some(0);
        af.width = Some(0);
        af.height = Some(0);
        af.accent_color = Some(Color::from_rgb(0, 0, 0));

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(!a.need_processing(&af, now));

        a
    };

    let ready_incomplete = {
        let (m, mut a, mut af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Ready;

        af.a_hash = Some(0);

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(a.need_processing(&af, now));

        a
    };

    let failed_fresh = {
        let (m, mut a, af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Failed;
        a.updated_at = now;

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(!a.need_processing(&af, now));

        a
    };

    let failed_stale = {
        let (m, mut a, af) = prepare_asset(&flake, "pending");

        a.state = AssetState::Failed;
        a.updated_at = now - Asset::TIME_BEFORE_REPROCESSING;

        insert_full_asset((&m, &a, &af), &repo).await?;

        assert!(a.need_processing(&af, now));

        a
    };

    // Gets a list of assets to process
    let for_processing = repo.get_for_processing(50).await?;

    // Checks the received list
    assert_eq!(for_processing.len(), 4);

    assert_eq!(for_processing[0].inner, pending);

    assert_eq!(for_processing[1].inner, processing_stale);
    assert!((now - for_processing[1].inner.updated_at) >= Asset::TIME_BEFORE_REPROCESSING);

    assert_eq!(for_processing[2].inner, ready_incomplete);
    assert!(!for_processing[2].features.enough_fields());

    assert_eq!(for_processing[3].inner, failed_stale);
    assert!((now - for_processing[3].inner.updated_at) >= Asset::TIME_BEFORE_REPROCESSING);

    let ids = for_processing.iter().map(|a| a.id()).collect::<Vec<_>>();

    assert!(!ids.contains(&processing_fresh.id));
    assert!(!ids.contains(&failed_fresh.id));
    assert!(!ids.contains(&ready_complete.id));

    Ok(())
}
