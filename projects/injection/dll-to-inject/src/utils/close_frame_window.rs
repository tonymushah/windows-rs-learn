use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows::Win32::{
    Foundation::{CloseHandle, HWND},
    UI::WindowsAndMessaging::CloseWindow,
};

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum CloseFrameWindowError {
    Handle(#[from] raw_window_handle::HandleError),
    Windows(#[from] windows_core::Error),
}

pub fn close_frame_window(frame: &eframe::Frame) -> Result<(), CloseFrameWindowError> {
    let handle = frame.window_handle()?.as_raw();
    match handle {
        RawWindowHandle::Win32(mut win32) => {
            let handle = HWND(&raw mut win32.hwnd as _);
            unsafe {
                CloseWindow(handle)?;
            }
        }
        RawWindowHandle::WinRt(mut winrt) => {
            let handle = HWND(&raw mut winrt.core_window as _);
            unsafe {
                CloseHandle(handle.into())?;
            }
        }
        _ => {}
    };
    Ok(())
}
