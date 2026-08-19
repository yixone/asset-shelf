use models::types::MediaId;
use result::Result;

use crate::{cleanup::delete, runtime::WorkerContext};

pub async fn remove_media_by_id(ctx: &WorkerContext, id: &MediaId) -> Result<()> {
    let media = ctx.db.media.get_by_id(id).await?;
    delete::delete_media(ctx, media).await
}
