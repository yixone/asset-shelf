use models::entities::Asset;

use crate::{
    core::{Result, result::InsertResult},
    ops::AssetsOps,
    sqlite::SqliteUnit,
};

impl<T> AssetsOps for T
where
    T: SqliteUnit,
{
    async fn insert_asset(&mut self, asset: &Asset) -> Result<InsertResult> {
        let res = sqlx::query(
            r#"
            INSERT INTO assets (
                id, state,
                media_id,
                created_at, deleted_at,
                title, caption, source_url,
                width, height, accent_color
            )
            VALUES (
                ?, ?,
                ?,
                ?, ?,
                ?, ?, ?,
                ?, ?, ?
            )
            "#,
        )
        .bind(asset.id)
        .bind(asset.state)
        .bind(&asset.media_id)
        .bind(asset.created_at)
        .bind(asset.deleted_at)
        .bind(&asset.title)
        .bind(&asset.caption)
        .bind(&asset.source_url)
        .bind(asset.width)
        .bind(asset.height)
        .bind(asset.accent_color)
        .execute(self.exec())
        .await?;
        Ok(res.into())
    }
}
