use windows::Win32::{
    Foundation::HMODULE,
    System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, MODULEENTRY32W, Module32FirstW, Module32NextW, TH32CS_SNAPMODULE,
    },
};
use windows_core::{HSTRING, Owned};

#[derive(Debug, Clone)]
pub struct ModuleEntry {
    pub handle: HMODULE,
    pub name: String,
    pub path: String,
}

pub fn modules(process_id: Option<u32>) -> windows_core::Result<Vec<ModuleEntry>> {
    let mut module32 = MODULEENTRY32W {
        dwSize: size_of::<MODULEENTRY32W>() as u32,
        ..Default::default()
    };
    let mut entries = Vec::<ModuleEntry>::new();
    let dbg_handle = unsafe {
        Owned::new(CreateToolhelp32Snapshot(
            TH32CS_SNAPMODULE,
            process_id.unwrap_or_default(),
        )?)
    };
    unsafe {
        Module32FirstW(*dbg_handle, &mut module32)?;
        while Module32NextW(*dbg_handle, &mut module32).is_ok() {
            entries.push(ModuleEntry {
                handle: module32.hModule,
                name: HSTRING::from_wide(&module32.szModule).to_string(),
                path: HSTRING::from_wide(&module32.szExePath).to_string(),
            });
        }
    }
    Ok(entries)
}
