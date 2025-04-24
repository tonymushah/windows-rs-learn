#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    NulFFI(#[from] std::ffi::NulError),
    Windows(#[from] windows_core::Error),
    StdIO(#[from] std::io::Error),
    #[error("Cannot transform an OsStr to an std &str")]
    OsStrToStdStr,
}
