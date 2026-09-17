/// Creates a newtype for the identifier
///
/// Automatically implements [`std::fmt::Display`] and [`From<INNER_TYPE>`] for the newtype
macro_rules! id_type {
    {$( #[$meta: meta] )* $id: ident as $id_ty: ty} => {
        $( #[$meta] )*
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $id(pub $id_ty);

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$id_ty> for $id {
            fn from(i: $id_ty) -> Self {
                $id(i)
            }
        }
    };
}

/// Creates a copyable newtype for the ID
///
/// In addition to the [`standard`](id_type) implementations,
/// it implements the [`Copy`] trait for the newtype
macro_rules! id_type_copy {
    ($( #[$meta: meta] )* $id: ident as $id_ty: ty) => {
        id_type!(#[derive(Copy)] $( #[$meta] )* $id as $id_ty);
    };
}

id_type_copy!(AssetId as snowflake::SnowflakeId);

id_type_copy!(CollectionId as snowflake::SnowflakeId);
id_type_copy!(CollectionAssetId as snowflake::SnowflakeId);

id_type_copy!(MediaFileId as snowflake::SnowflakeId);
id_type!(MediaId as snowflake::SnowflakeIdStr);
