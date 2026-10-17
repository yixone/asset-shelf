//! Domain types and structures used in the application
//!
//! Types defined in this crate represent the application's core concepts
//! and are primarily used used as internal types
//!
//! ## Integration with the Rust crates ecosystem:
//!
//! Support for third-party crates is optional and enabled via dependency features:
//!
//! - `sqlx`: Enables [`sqlx::Type`]
//!   implementations for types defined in this crate
//! - `serde`: Enables [`serde::Serialize`] and [`serde::Deserialize`]
//!   implementations for supported types, excluding domain entities

pub mod result;

pub mod id;

pub mod entities;
pub mod patches;
pub mod types;

/*
    // * Temp сам удаляет файл при Drop до коммита
    let temp = ctx.storage.upload_stream(stream, key, callback).await?;

    // Создает модель `Asset`
    let asset = Asset::new(...);

    // Создает модель `FileGroup`
    let mut file_group = FileGroup::new(...);

    // Создает модель `File` с вариантом `Original`
    let file = File::new(file_group.id(), FileVariant::Original, ...);

    // `create` требует (Asset, FileGroup и File), которые он атомарно вставит
    let created = ctx.db.assets.create(asset, file_group, file).await;

    // Перемещает файл в постоянное хранилище
    if let Err(e) = temp.persist().await? {
        ctx.db.assets.set_offline(asset.id()).await?;
        return Err(e);
    }

    // Note:
    // Если перемещение файла в хранилище упадет и упадет установка `set_offline`,
    // при следующей попытке получить оригинал сервис должен сам попробовать установить `is_offline = true`
*/

/*
    // * Temp сам удаляет файл при Drop до коммита
    let temp = ctx.storage.upload_stream(stream, key, callback).await?;

    // Вставляет файл группы в бд с проверкой
    let created = ctx.db.files.insert_file_fn(group_id, |group| {
        if group.has_file(FileVariant::Thumbnail) {
            return ...;
        }

        let file = File::new(group.id(), FileVariant::Thumbnail, ...);

        Ok(file)
    }).await?;

    // Перемещает файл в постоянное хранилище
    if let Err(e) = temp.persist().await? {
        ctx.db.files.set_ofline(created.key()).await?;
        return Err(e);
    }
*/
