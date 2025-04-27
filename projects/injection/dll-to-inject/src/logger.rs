use std::fs::create_dir_all;
use std::io;
use std::time::SystemTime;
use std::{env::temp_dir, fs::File};

use fern::{Dispatch, InitError};
use humantime::format_rfc3339_seconds;

fn get_log_file() -> io::Result<File> {
    let temp_dir = temp_dir();
    let temp_dir = {
        let dir = temp_dir.join("win-rs-learn-dll-injection");
        let _ = create_dir_all(&dir);
        dir
    };
    File::create(
        temp_dir
            .join(format!("_{}_.log", format_rfc3339_seconds(SystemTime::now())).replace(":", "_")),
    )
}

pub fn setup_log() -> Result<(), InitError> {
    let mut dispatch = Dispatch::new();
    if let Ok(d) = get_log_file() {
        dispatch = dispatch.chain(
            Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!(
                        "[{} {} {}] {}",
                        humantime::format_rfc3339_seconds(SystemTime::now()),
                        record.level(),
                        record.target(),
                        message
                    ))
                })
                .chain(d),
        );
    }
    dispatch
        .chain(
            Dispatch::new()
                .level(log::LevelFilter::Debug)
                .chain(Dispatch::new().chain(crate::run::dis_log::output())),
        )
        .apply()?;
    Ok(())
}
