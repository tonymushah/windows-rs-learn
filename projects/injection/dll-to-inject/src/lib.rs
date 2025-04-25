mod main_work;
mod run;
pub mod utils;

use std::{os::raw::c_void, panic};

use main_work::{init_work, wait_for_work};
use utils::message_box;
use windows::Win32::{
    Foundation::HMODULE,
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
};
use windows_core::BOOL;

fn set_panic_hook() {
    panic::set_hook(Box::new(|d| {
        message_box("Panic!", {
            if let Some(d) = d.payload().downcast_ref::<String>() {
                d.as_str()
            } else if let Some(d) = d.payload().downcast_ref::<&'static str>() {
                d
            } else {
                "..."
            }
        });
    }));
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
            set_panic_hook();
            message_box("Attached", "Your dll has been injected");
            init_work(_hmodule);
        }
        DLL_PROCESS_DETACH => {
            wait_for_work(true);
            message_box("Dettached", "Dettached dll");
        }
        _ => {}
    }
    BOOL::from(true)
}
