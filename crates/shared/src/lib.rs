use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub enum SlaError {
    DatabaseError(String),
    ValidationError(String),
    NotFound(String),
    InternalError(String),
}

impl std::fmt::Display for SlaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SlaError::DatabaseError(s) => write!(f, "Database error: {}", s),
            SlaError::ValidationError(s) => write!(f, "Validation error: {}", s),
            SlaError::NotFound(s) => write!(f, "Not found: {}", s),
            SlaError::InternalError(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl std::error::Error for SlaError {}
