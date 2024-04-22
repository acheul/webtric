use thiserror::Error;

/// webtric's Result type
pub type Result<T> = std::result::Result<T, Error>;

/// webtric's Error type
/// 
/// It wouldn't be used very frequently.
#[derive(Error, Debug)]
pub enum Error {
  #[error("webtric error: {0}")]
  Msg(String),
  /// for ignore-able errors
  #[error("Ignore")]
  Ignore,
}