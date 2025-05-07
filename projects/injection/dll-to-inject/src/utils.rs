pub mod close_frame_window;
pub mod get_heap_blocks;
pub mod get_loaded_module;

use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MessageBoxW};
use windows_core::{HSTRING, PCWSTR};

pub fn message_box<T, C>(caption: &C, text: &T)
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
