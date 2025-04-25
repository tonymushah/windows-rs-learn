use crate::utils::message_box;
use windows::Win32::{
    Foundation::HMODULE,
    System::Threading::{
        /*CreateThread,*/ CreateThreadpoolWork, PTP_CALLBACK_INSTANCE, PTP_WORK,
        SubmitThreadpoolWork, WaitForThreadpoolWorkCallbacks,
    },
};
// use windows_core::Owned;

use std::{os::raw::c_void, sync::RwLock};

static WORK: RwLock<Option<PTP_WORK>> = RwLock::new(None);

pub fn wait_for_work(cancel: bool) {
    WORK.clear_poison();
    let mut write_work = WORK.write().unwrap();
    if let Some(work) = write_work.take() {
        unsafe { WaitForThreadpoolWorkCallbacks(work, cancel) };
    }
}

extern "system" fn run_work(_pfnwk: PTP_CALLBACK_INSTANCE, _ctx: *mut c_void, _work: PTP_WORK) {
    // let _work = unsafe { Owned::new(_work) };
    if let Err(err) = crate::run::run(HMODULE(_ctx)) {
        message_box("Cannot execute payload", &err.to_string());
    }
    WORK.clear_poison();
    WORK.write().unwrap().take();
    //Idk why the window just froze when this message box is not here.
    message_box("Killing work", "The eframe window is now closed");
}

pub fn init_work(_hmodule: HMODULE) {
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
