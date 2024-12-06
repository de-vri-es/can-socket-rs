use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("Failed to access file")]
    IO(#[from] std::io::Error),
    #[error("EDS syntax error: {0}")]
    SyntaxError(String),
}

impl<'a> From<&'a str> for LoadError {
    fn from(value: &'a str) -> Self {
        Self::SyntaxError(value.to_string())
    }
}

impl From<String> for LoadError {
    fn from(value: String) -> Self {
        Self::SyntaxError(value)
    }
}
