pub mod utils;

use std::{fmt::Write, fs::read_dir, os::raw::c_void, path::PathBuf};

use utils::message_box;
use windows::Win32::{
    Foundation::HMODULE,
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};
use windows_core::BOOL;

fn run() -> anyhow::Result<()> {
    Ok(())
}

#[allow(non_snake_case, clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn DllMain(
    _hmodule: HMODULE,
    ul_reason_for_call: u32,
    _lpReserved: *mut c_void,
) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            message_box("Attached", "Your dll has been injected");
            if let Err(err) = run() {
                message_box("Cannot execute payload", &err.to_string());
            }
        }
        DLL_PROCESS_DETACH => message_box("Dettached", "Dettached dll"),
        _ => {}
    }
    BOOL::from(true)
}
