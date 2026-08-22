use crate::models::{HistogramBucket, HistogramBucketBound};

/// Calculates percentiles for a set of histogram buckets
pub fn calc_buckets_percentile(
    p: f64,
    total: u64,
    buckets: &[HistogramBucket],
) -> HistogramBucketBound {
    let bound = (total as f64 * p).ceil() as u64;
    for b in buckets {
        if b.count >= bound {
            return b.upper_bound;
        }
    }

    HistogramBucketBound::Inf
}
