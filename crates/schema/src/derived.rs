/// Adds feature-gated derives and attributes to the wrapped type item
macro_rules! auto_derived_ty {
    (no_sqlx; $item:item) => {
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        $item
    };

    ($item:item) => {
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(
            feature = "serde",
            derive(serde::Serialize, serde::Deserialize),
            serde(rename_all = "snake_case")
        )]
        $item
    };
}
