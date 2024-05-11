use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitHydraError {
    #[error("Generic Error message: {error}")]
    GenErr {
        error: String
    }
}

pub fn new_gh_err(val: impl Into<String>) -> GitHydraError {
    GitHydraError::GenErr { error: val.into() }
}
