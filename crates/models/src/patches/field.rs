/// Data field for the patch
#[derive(Debug, Default, PartialEq)]
pub enum PatchField<T> {
    /// Change the value of the current field
    Set(T),

    #[default]
    /// Ignore field update
    Unset,
}

impl<T> PatchField<T> {
    pub fn is_set(&self) -> bool {
        matches!(self, PatchField::Set(..))
    }

    pub fn is_unset(&self) -> bool {
        matches!(self, PatchField::Unset)
    }

    pub fn set(&self) -> Option<&T> {
        match self {
            PatchField::Set(t) => Some(t),
            PatchField::Unset => None,
        }
    }
}

impl From<Option<String>> for PatchField<Option<String>> {
    fn from(str: Option<String>) -> Self {
        match str {
            Some(s) => {
                if s.trim().is_empty() {
                    PatchField::Set(None)
                } else {
                    PatchField::Set(Some(s))
                }
            }
            None => PatchField::Unset,
        }
    }
}

impl From<String> for PatchField<Option<String>> {
    fn from(str: String) -> Self {
        if str.trim().is_empty() {
            PatchField::Set(None)
        } else {
            PatchField::Set(Some(str))
        }
    }
}

impl<T> From<Option<T>> for PatchField<T> {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => PatchField::Set(v),
            None => PatchField::Unset,
        }
    }
}
