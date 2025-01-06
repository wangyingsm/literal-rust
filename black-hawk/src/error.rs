use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("skip newline error")]
    SkipNewlineError,
    #[error("parse header error")]
    ParseHeaderError,
}
