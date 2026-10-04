macro_rules! patch {
    (
        $( #[$meta: meta] )*
        $model_name: ident {
            $(
                $( #[$f_meta: meta] )*
                $f_id: ident: $f_ty: ty
            ),+ $(,)?
        }, $domain: path
    ) => {
        $( #[$meta] )*
        #[derive(Debug, Default)]
        pub struct $model_name {
            $(
                $( #[$f_meta] )*
                pub $f_id: $crate::patches::PatchField<$f_ty>
            ),+
        }

        impl $model_name {
            /// Creates an empty patch
            pub fn new() -> Self {
                Self::default()
            }

            $(
                pub fn $f_id(mut self, $f_id: $f_ty) -> Self {
                    self.$f_id = $crate::patches::PatchField::Set($f_id);
                    self
                }
            )*

            /// Returns the number of fields being updated
            pub fn changes(&self) -> usize {
                let mut changes = 0;
                $(
                    if self.$f_id.is_set() {
                        changes += 1;
                    }
                )*
                changes
            }

            /// Applies the model patch to the domain model
            pub fn apply_domain(self, domain: &mut $domain) {
                $(
                    if let crate::patches::PatchField::Set(v) = self.$f_id {
                        domain.$f_id = v;
                    }
                )*
            }

            /// Appends changed fields to the SQL `SET` clause
            #[cfg(feature = "sqlx")]
            pub fn apply_sql<'a, DB>(&'a self, qb: &mut sqlx::QueryBuilder<'a, DB>)
            where
                DB: sqlx::Database,
                $(
                    $f_ty: sqlx::Encode<'a, DB> + sqlx::Type<DB>
                ),+
            {
                let mut sep = qb.separated(",");
                $(
                    if let crate::patches::PatchField::Set(v) = &self.$f_id {
                        sep.push(concat!(stringify!($f_id), " = "));
                        sep.push_bind_unseparated(v);
                    }
                )*
            }
        }
    };
}
