/// Statistics describing the capacity and free space of a file system
#[derive(Debug, Clone)]
pub struct DiskUsageStats {
    /// Number of unused bytes in the file system
    b_free: u64,
    /// Total capacity of the file system in bytes
    b_total: u64,
}

impl DiskUsageStats {
    /// Creates a new [`DiskUsageStats`]
    pub fn new(b_free: u64, b_total: u64) -> Self {
        Self { b_free, b_total }
    }

    /// Returns the number of unused bytes in the file system
    pub fn bytes_free(&self) -> u64 {
        self.b_free
    }

    /// Returns the number of bytes currently occupied in the file system
    #[inline]
    pub fn bytes_used(&self) -> u64 {
        self.bytes_total() - self.bytes_free()
    }

    /// Returns the total capacity of the file system in bytes
    pub fn bytes_total(&self) -> u64 {
        self.b_total
    }
}
