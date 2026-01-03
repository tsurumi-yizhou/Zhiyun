use thiserror::Error;

#[derive(Debug, Error)]
pub enum EndpointError {
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Context window exceeded: limit {limit}, requested {requested}")]
    ContextWindowExceeded { limit: u32, requested: u32 },

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type EndpointResult<T> = Result<T, EndpointError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EndpointError::ContextWindowExceeded {
            limit: 4096,
            requested: 5000,
        };
        assert!(err.to_string().contains("limit 4096"));
        assert!(err.to_string().contains("requested 5000"));
    }
}
