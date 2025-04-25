use std::{fmt::Write, fs::read_dir, os::raw::c_void, path::PathBuf};

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

fn run() -> anyhow::Result<()> {
    let path = PathBuf::from(".");
    let d = read_dir(path.canonicalize()?)?;
    let entrys = d
        .flatten()
        .flat_map(|d| d.path().to_str().map(String::from))
        .collect::<Vec<_>>();

    let mut buf = String::new();
    for entry in &entrys {
        let _ = writeln!(&mut buf, "{entry}");
    }

    message_box(&format!("Found {} entries:", entrys.len()), &buf);

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
