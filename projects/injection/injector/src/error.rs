#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    NulFFI(#[from] std::ffi::NulError),
    Windows(#[from] windows_core::Error),
}
