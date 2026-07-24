macro_rules! pattern_matcher {
    ($val: expr, $ex: expr, $idx: expr, ? $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex,
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr, $pattern: literal $(, $tail: tt)*) => {
        pattern_matcher!(
            $val,
            $ex && ($val[$idx] == $pattern),
            $idx + 1
            $(,$tail)*
        )
    };
    ($val: expr, $ex: expr, $idx: expr) => {
        $val.len() >= ($idx) && $ex
    };
}

macro_rules! define_mimes {
    (
        $(
            $type_id: ident,
            $mimetype: literal,
            [ $( ( $offset: expr, [ $( $tokens:tt ),* ] ) ),+ $(,)? ]
        );* $(;)?
    ) => {
        /// Media Content Type
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum MimeType {
            $(
                #[cfg_attr(feature = "sqlx", sqlx(rename = $mimetype))]
                #[cfg_attr(feature = "serde",serde(rename = $mimetype))]
                $type_id
            ),*
        }
        impl MimeType {
            pub const fn as_str(&self) -> &'static str {
                match self {
                    $(Self::$type_id => $mimetype),*
                }
            }
        }

        $(
            #[allow(non_upper_case_globals)]
            const $type_id: &'static MimePattern = &MimePattern {
                mime: MimeType::$type_id,
                matcher: |b| $( pattern_matcher!(b, !b.is_empty(), $offset, $($tokens),*) )||+,
            };
        )+
        const PATTERNS: &[&MimePattern] = &[
            $( $type_id ),+
        ];
    };
}

macro_rules! generate_ptree {
    (
        $(
            $prefix: literal => [
                $( $mime_pattern: expr ),+
            ] as $bucket: ident;
        )+
    ) => {
        $(
            const $bucket: &[&MimePattern] = &[$( $mime_pattern ),+];
        )+

        static PREFIX_TREE: [&[&MimePattern]; 256] = {
            const EMPTY: &[&MimePattern] = &[];
            let mut arr = [EMPTY;256];
            $(
                arr[$prefix] = $bucket;
            )+
            arr
        };
    };
}
