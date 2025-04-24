use std::ffi::{CStr, CString};

use windows::Win32::System::Diagnostics::ToolHelp::*;
use windows_core::Owned;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
enum GetProcessIdError {
    Null(#[from] std::ffi::NulError),
    Windows(#[from] windows_core::Error),
}

fn get_process_id<T>(target: &T) -> Result<u32, GetProcessIdError>
where
    T: AsRef<str> + ?Sized,
{
    let mut entry = PROCESSENTRY32 {
        dwSize: size_of::<PROCESSENTRY32>() as u32,
        ..Default::default()
    };
    let target_c_string = CString::new(target.as_ref())?;
    unsafe {
        let handle = Owned::new(CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)?);
        Process32First(*handle, &mut entry)?;
        loop {
            Process32Next(*handle, &mut entry)?;
            let process_exe = CStr::from_ptr(entry.szExeFile.as_ptr());
            if process_exe == target_c_string.as_c_str() {
                return Ok(entry.th32ProcessID);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    println!("notepad.exe [{}]", get_process_id("notepad.exe")?);
    Ok(())
}
