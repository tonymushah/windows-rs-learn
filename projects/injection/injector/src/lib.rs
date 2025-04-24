pub mod debug_priv;
pub mod error;
pub mod injection;
pub mod process;

pub(crate) type Result<T, E = error::Error> = std::result::Result<T, E>;
