// LLM Endpoint Error Types
use thiserror::Error;

/// Errors that can occur in the endpoint module
#[derive(Debug, Error)]
pub enum EndpointError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication failed for provider: {provider_id}")]
    AuthenticationFailed { provider_id: String },

    #[error("Rate limit exceeded for provider: {provider_id}")]
    RateLimitExceeded {
        provider_id: String,
        retry_after: Option<u64>,
    },

    #[error("Model not available: {provider_id}/{model_id}")]
    ModelNotAvailable {
        provider_id: String,
        model_id: String,
    },

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Stream error: {0}")]
    StreamError(String),

    #[error("Fetch error: {0}")]
    FetchError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("File error: {0}")]
    FileError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EndpointError::NetworkError("connection refused".to_string());
        assert_eq!(err.to_string(), "Network error: connection refused");
    }

    #[test]
    fn test_authentication_failed() {
        let err = EndpointError::AuthenticationFailed {
            provider_id: "openai".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Authentication failed for provider: openai"
        );
    }

    #[test]
    fn test_rate_limit_exceeded() {
        let err = EndpointError::RateLimitExceeded {
            provider_id: "openai".to_string(),
            retry_after: Some(60),
        };
        assert_eq!(err.to_string(), "Rate limit exceeded for provider: openai");
    }

    #[test]
    fn test_model_not_available() {
        let err = EndpointError::ModelNotAvailable {
            provider_id: "openai".to_string(),
            model_id: "gpt-5".to_string(),
        };
        assert_eq!(err.to_string(), "Model not available: openai/gpt-5");
    }

    #[test]
    fn test_provider_not_found() {
        let err = EndpointError::ProviderNotFound("unknown".to_string());
        assert_eq!(err.to_string(), "Provider not found: unknown");
    }

    #[test]
    fn test_invalid_request() {
        let err = EndpointError::InvalidRequest("empty messages".to_string());
        assert_eq!(err.to_string(), "Invalid request: empty messages");
    }

    #[test]
    fn test_stream_error() {
        let err = EndpointError::StreamError("connection closed".to_string());
        assert_eq!(err.to_string(), "Stream error: connection closed");
    }

    #[test]
    fn test_fetch_error() {
        let err = EndpointError::FetchError("HTTP 404".to_string());
        assert_eq!(err.to_string(), "Fetch error: HTTP 404");
    }

    #[test]
    fn test_parse_error() {
        let err = EndpointError::ParseError("invalid JSON".to_string());
        assert_eq!(err.to_string(), "Parse error: invalid JSON");
    }

    #[test]
    fn test_api_error() {
        let err = EndpointError::ApiError("server error".to_string());
        assert_eq!(err.to_string(), "API error: server error");
    }

    #[test]
    fn test_file_error() {
        let err = EndpointError::FileError("upload failed".to_string());
        assert_eq!(err.to_string(), "File error: upload failed");
    }

    #[test]
    fn test_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: EndpointError = io_err.into();
        assert!(err.to_string().contains("IO error"));
    }
}

