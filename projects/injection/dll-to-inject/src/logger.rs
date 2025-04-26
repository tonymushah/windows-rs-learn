use fern::{Dispatch, InitError};

pub fn setup_log() -> Result<(), InitError> {
    Dispatch::new()
        .chain(crate::run::dis_log::output())
        .apply()?;
    Ok(())
}
