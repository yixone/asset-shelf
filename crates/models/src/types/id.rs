use flake_id::{FlakeId, str::FlakeIdStr};

macro_rules! id_type {
    (
        $( #[$meta: meta] )*
        $id: ident as $id_ty: ty
    ) => {
        #[derive(Debug, Clone, Eq, PartialEq, Hash)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $( #[$meta] )*
        pub struct $id(pub $id_ty);

        impl std::fmt::Display for $id {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }

        impl From<$id_ty> for $id {
            fn from(i: $id_ty) -> Self{
                $id(i)
            }
        }
    };
}

id_type!(
    #[derive(Copy)]
    AssetId as FlakeId
);

id_type!(MediaId as FlakeIdStr);
id_type!(MediaFileId as FlakeId);

id_type!(
    #[derive(Copy)]
    CollectionId as FlakeId
);
id_type!(
    #[derive(Copy)]
    CollectionAssetId as FlakeId
);

impl From<FlakeId> for MediaId {
    fn from(id: FlakeId) -> Self {
        MediaId(id.into())
    }
}
