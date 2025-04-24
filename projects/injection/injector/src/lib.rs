pub mod error;
pub mod process;

pub(crate) type Result<T, E = error::Error> = std::result::Result<T, E>;
