use std::os::raw::c_void;

use windows::Win32::{
    Foundation::HMODULE,
    System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
    UI::WindowsAndMessaging::{MB_ICONINFORMATION, MessageBoxW},
};
use windows_core::{BOOL, HSTRING, PCWSTR};

fn message_box<T, C>(caption: &C, text: &T)
where
    T: AsRef<str> + ?Sized,
    C: AsRef<str> + ?Sized,
{
    let h_str_text = HSTRING::from(text.as_ref());
    let h_str_caption = HSTRING::from(caption.as_ref());
    unsafe {
        MessageBoxW(
            None,
            PCWSTR::from_raw(h_str_text.as_ptr()),
            PCWSTR::from_raw(h_str_caption.as_ptr()),
            MB_ICONINFORMATION,
        );
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "system" fn DLLMain(
    _hmodule: HMODULE,
    ul_reason_for_call: u32,
    _lpReserved: *mut c_void,
) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => message_box("Attached", "Your dll has been injected"),
        DLL_PROCESS_DETACH => message_box("Dettached", "Dettached dll"),
        _ => {}
    }
    BOOL::from(true)
}
