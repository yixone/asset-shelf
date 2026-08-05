use actix_web::{
    HttpRequest,
    http::header::{Header, Range},
};

/// Parses the `range` header from the HTTP request and returns it as a `(u64, u64)` tuple
///
/// Returns None if the request does not contain the header
/// or the header is specified in a non-standard format
pub fn parse_request_range(r: &HttpRequest, full_length: u64) -> Option<(u64, u64)> {
    if let Ok(range) = Range::parse(r) {
        match range {
            Range::Bytes(range) if range.len() == 1 => {
                if let Some(b) = range.first() {
                    if let Some((f, t)) = b.to_satisfiable_range(full_length) {
                        Some((f, t))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    } else {
        None
    }
}
