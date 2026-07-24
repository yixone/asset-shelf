use models::{entities::AssetState, types::Color};

/// Data field for the patch
#[derive(Debug, Default, PartialEq)]
pub enum PatchField<T> {
    /// Change the value of the current field
    Set(T),

    /// Ignore field update
    #[default]
    Ignore,
}

impl From<Option<String>> for PatchField<Option<String>> {
    fn from(v: Option<String>) -> Self {
        match v {
            Some(v) => {
                if v.trim().is_empty() {
                    PatchField::Set(None)
                } else {
                    PatchField::Set(Some(v))
                }
            }
            None => PatchField::Ignore,
        }
    }
}

impl<T> From<Option<T>> for PatchField<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => PatchField::Set(v),
            None => PatchField::Ignore,
        }
    }
}

macro_rules! patch_model {
    {
        $( #[$meta: meta] )*
        $model_name: ident {
            $(
                $( #[$f_meta: meta] )*
                $f_id: ident: $f_ty: ty
            ),+ $(,)?
        }
    } => {
         $( #[$meta] )*
        #[derive(Debug, Default)]
        pub struct $model_name {
            $(
                $( #[$f_meta] )*
                pub $f_id: crate::types::PatchField<$f_ty>
            ),+
        }

        impl $model_name {
            pub fn new() -> Self {
                Self::default()
            }

            $(
                pub fn $f_id(mut self, $f_id: $f_ty) -> Self {
                    self.$f_id = crate::types::PatchField::Set($f_id);
                    self
                }
            )*

            /// Returns the number of fields being updated
            pub fn changes(&self) -> usize {
                let mut changes = 0;
                $(
                    if let crate::types::PatchField::Set(_) = &self.$f_id {
                        changes += 1;
                    }
                )*
                changes
            }

            /// Applies the model patch to the SQL query
            pub fn apply_sql<'a, DB>(&'a self, qb: &mut sqlx::QueryBuilder<'a, DB>)
            where
                DB: sqlx::Database,
                $(
                    $f_ty: sqlx::Encode<'a, DB> + sqlx::Type<DB>
                ),+
            {
                let mut sep = qb.separated(",");
                $(
                    if let crate::types::PatchField::Set(v) = &self.$f_id {
                        sep.push(concat!(stringify!($f_id), " = "));
                        sep.push_bind_unseparated(v);
                    }
                )*
            }
        }
    };
}

patch_model! {
    AssetPatch {
        state: AssetState,
        title: Option<String>,
        caption: Option<String>,
        source_url: Option<String>,
    }
}

patch_model! {
    AssetFeaturesPatch {
        p_hash: Option<i64>,
        a_hash: Option<i64>,
        width: Option<u32>,
        height: Option<u32>,
        accent_color: Option<Color>
    }
}
