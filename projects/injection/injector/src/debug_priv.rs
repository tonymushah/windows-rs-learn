use windows::Win32::{
    Foundation::*,
    Security::{
        AdjustTokenPrivileges, LookupPrivilegeValueW, SE_DEBUG_NAME, SE_PRIVILEGE_ENABLED,
        TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES, TOKEN_QUERY,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};
use windows_core::{Owned, PCWSTR};

pub fn enable_debug_priv() -> windows_core::Result<()> {
    let mut h_token = unsafe { Owned::new(HANDLE::default()) };
    let mut lpluid = LUID::default();
    let mut token_privileges = TOKEN_PRIVILEGES::default();

    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut *h_token,
        )?;
        LookupPrivilegeValueW(PCWSTR::default(), SE_DEBUG_NAME, &mut lpluid)?;

        token_privileges.PrivilegeCount = 1;
        token_privileges.Privileges[0].Luid = lpluid;
        token_privileges.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;

        AdjustTokenPrivileges(
            *h_token,
            false,
            Some(&token_privileges),
            size_of_val(&token_privileges) as u32,
            None,
            None,
        )?;
    }
    Ok(())
}
