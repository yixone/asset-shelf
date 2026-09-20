auto_derived_ty! {
    no_sqlx;

    /// Specifies the field used to sort assets
    #[derive(Debug)]
    pub enum SortOrder {
        /// Ascending order
        Asc,
        /// Descending order
        Desc
    }
}
