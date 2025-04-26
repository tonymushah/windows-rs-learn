use fern::{Dispatch, InitError};

pub fn setup_log() -> Result<(), InitError> {
    Dispatch::new()
        .level(log::LevelFilter::Debug)
        .chain(Dispatch::new().chain(crate::run::dis_log::output()))
        .apply()?;
    Ok(())
}
