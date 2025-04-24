use std::ffi::CString;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{MB_OK, MessageBoxA, MessageBoxW},
};
use windows_core::{HSTRING, PCSTR, PCWSTR};

fn main() {
    let lptext = "sdffdsfdsTony";

    let lpcaption = "AAAaaaaaaaaaaaaaadsads sadsad a";

    unsafe {
        {
            let lptext = CString::new(lptext).unwrap();
            let lpcaption = CString::new(lpcaption).unwrap();
            let res = MessageBoxA(
                None,
                PCSTR::from_raw(lptext.as_ptr() as *const u8),
                PCSTR::from_raw(lpcaption.as_ptr() as *const u8),
                MB_OK,
            );
            println!("{:?}", res);
        }
{
        let lptext = HSTRING::from(lptext);
        let lpcaption = HSTRING::from(lpcaption);
        let res = MessageBoxW(
            None,
            PCWSTR::from_raw(lptext.as_ptr()),
            PCWSTR::from_raw(lpcaption.as_ptr()),
            MB_OK,
        );
        println!("{:?}", res);}
    };
}
