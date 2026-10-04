#[macro_use]
mod _macro;

mod field;
pub use field::PatchField;

use crate::entities::File;

patch! {
    FilePatch {
        duration_ms: Option<i64>
    },
    File
}
