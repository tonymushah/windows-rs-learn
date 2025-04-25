mod run;
pub mod utils;

use std::{os::raw::c_void, panic, sync::RwLock};

use utils::message_box;
use windows::Win32::{
    Foundation::HMODULE,
    System::{
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        Threading::{
            /*CreateThread,*/ CreateThreadpoolWork, PTP_CALLBACK_INSTANCE, PTP_WORK,
            SubmitThreadpoolWork, WaitForThreadpoolWorkCallbacks,
        },
    },
};
use windows_core::BOOL;

static WORK: RwLock<Option<PTP_WORK>> = RwLock::new(None);

extern "system" fn run_work(_pfnwk: PTP_CALLBACK_INSTANCE, _ctx: *mut c_void, _work: PTP_WORK) {
    if let Err(err) = run::run(HMODULE(_ctx)) {
        message_box("Cannot execute payload", &err.to_string());
    }
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
            message_box("Attached", "Your dll has been injected");
            WORK.clear_poison();
            let mut write_work = WORK.write().unwrap();
            // CreateThread(lpthreadattributes, dwstacksize, lpstartaddress, lpparameter, dwcreationflags, lpthreadid)
            if write_work.is_none() {
                match unsafe { CreateThreadpoolWork(Some(run_work), Some(_hmodule.0), None) } {
                    Ok(work) => {
                        unsafe { SubmitThreadpoolWork(work) };
                        write_work.replace(work);
                    }
                    Err(err) => {
                        message_box("Cannot create thread work", &err.message());
                    }
                }
            }
        }
        DLL_PROCESS_DETACH => {
            message_box("Dettached", "Dettached dll");
            WORK.clear_poison();
            let mut write_work = WORK.write().unwrap();
            if let Some(work) = write_work.take() {
                unsafe { WaitForThreadpoolWorkCallbacks(work, true) };
            }
        }
        _ => {}
    }
    BOOL::from(true)
}
