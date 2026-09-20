auto_derived_ty! {
    /// Variant of a media file
    ///
    /// May represent the original media file or a generated
    /// derivative optimized for a specific use case
    #[derive(Debug)]
    pub enum MediaVariant {
        /// Original media file uploaded by the user
        Original,

        /// Low-resolution preview for quick asset display
        Thumbnail,

        /// Short looped video preview generated from the original video
        LoopPreview
    }
}
