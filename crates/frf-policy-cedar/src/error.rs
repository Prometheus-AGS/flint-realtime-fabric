use thiserror::Error;

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CedarError {
    #[error("policy parse error: {0}")]
    PolicyParse(String),
}
