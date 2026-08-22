#[derive(Debug)]
pub enum AdapterError {
    Prometheus(prometheus::Error),
    Io(std::io::Error),
    UnnamedMetric,
    EmptyMetric,
}

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for AdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AdapterError::Prometheus(error) => Some(error),
            AdapterError::Io(error) => Some(error),
            AdapterError::UnnamedMetric => None,
            AdapterError::EmptyMetric => None,
        }
    }
}

impl From<prometheus::Error> for AdapterError {
    fn from(err: prometheus::Error) -> Self {
        AdapterError::Prometheus(err)
    }
}

impl From<std::io::Error> for AdapterError {
    fn from(err: std::io::Error) -> Self {
        AdapterError::Io(err)
    }
}
