pub mod error;

#[cfg(feature = "actix")]
pub mod actix;

pub use error::{Error, ErrorKind};

pub type Result<T> = std::result::Result<T, Error>;

/// Creates a new error with the specified kind
#[macro_export]
macro_rules! create_error {
    ($kind: ident $( $tt:tt )?) => {
        $crate::Error::new( $crate::ErrorKind::$kind $( $tt )?)
    };

    (source = $source: expr) => {
        $crate::Error::internal($source)
    };
}
