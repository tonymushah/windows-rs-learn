use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    path::Path,
};

use windows::Win32::{
    Foundation::{HANDLE, HMODULE},
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::{GetModuleHandleA /*LoadLibraryA*/},
        Memory::{
            MEM_COMMIT, MEM_RELEASE, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
            VIRTUAL_ALLOCATION_TYPE, VirtualAllocEx, VirtualFreeEx,
        },
        Threading::{
            CreateRemoteThread, INFINITE, /*LPFIBER_START_ROUTINE,*/ LPTHREAD_START_ROUTINE,
            OpenProcess, PROCESS_ALL_ACCESS, WaitForSingleObject,
        },
    },
};
use windows_core::PCSTR;

use crate::process::get_process_id;

pub struct Process {
    handle: HANDLE,
}

/*
unsafe extern "system" fn load_library_a(parameter: *mut c_void) -> u32 {
    if !parameter.is_null() {
        match unsafe { LoadLibraryA(PCSTR(parameter as _)) } {
            Ok(_mod) => {}
            Err(_err) => {
                // eprintln!("{err}");
            }
        };
    } else {
        // println!("Null");
    }

    1
}
*/

unsafe fn get_load_library_a() -> crate::Result<LPTHREAD_START_ROUTINE> {
    windows_link::link!("kernel32.dll" "system" fn GetProcAddress(hmodule : HMODULE, lpprocname : windows_core::PCSTR) -> LPTHREAD_START_ROUTINE);

    let kernel_32_str = CString::new("Kernel32.dll")?;
    let kernel_32_pcstr = PCSTR::from_raw(kernel_32_str.as_ptr() as _);

    let load_lib_a_str = CString::new("LoadLibraryA")?;
    let load_lib_a_pcstr = PCSTR::from_raw(load_lib_a_str.as_ptr() as _);
    unsafe {
        let kernel_32 = GetModuleHandleA(kernel_32_pcstr)?;

        Ok(GetProcAddress(kernel_32, load_lib_a_pcstr))
    }
}

struct OwnedVirtualAllocEx<'a> {
    ptr: *mut c_void,
    process: &'a Process,
    path: &'a CStr,
}

impl<'a> OwnedVirtualAllocEx<'a> {
    unsafe fn new(process: &'a Process, path: &'a CStr) -> windows_core::Result<Self> {
        unsafe { Self::new_with_parameter(process, path, MEM_COMMIT, PAGE_READWRITE) }
    }
    unsafe fn new_with_parameter(
        process: &'a Process,
        path: &'a CStr,
        flallocationtype: VIRTUAL_ALLOCATION_TYPE,
        flprotect: PAGE_PROTECTION_FLAGS,
    ) -> windows_core::Result<Self> {
        let ptr = unsafe {
            VirtualAllocEx(
                process.handle,
                None,
                path.count_bytes() + 1,
                flallocationtype,
                flprotect,
            )
        };
        Ok(Self { ptr, process, path })
    }
    unsafe fn write_process(
        &self,
        lpnumberofbyteswritten: Option<*mut usize>,
    ) -> windows_core::Result<()> {
        unsafe {
            WriteProcessMemory(
                self.process.handle,
                self.ptr,
                self.path.as_ptr() as _,
                self.path.count_bytes() + 1,
                lpnumberofbyteswritten,
            )?;
        }
        Ok(())
    }
}

impl Drop for OwnedVirtualAllocEx<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = VirtualFreeEx(
                self.process.handle,
                self.ptr,
                self.path.count_bytes() + 1,
                MEM_RELEASE,
            );
        }
    }
}

impl Process {
    pub fn open(pid: u32) -> crate::Result<Self> {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, pid) }?;
        Ok(Self { handle })
    }
    pub fn open_by_exe_name<T>(target: &T) -> crate::Result<Self>
    where
        T: AsRef<str> + ?Sized,
    {
        let pid = get_process_id(target)?;
        Self::open(pid)
    }
    fn virtual_alloc<'a>(
        &'a self,
        target: &'a CStr,
    ) -> windows_core::Result<OwnedVirtualAllocEx<'a>> {
        unsafe { OwnedVirtualAllocEx::new(self, target) }
    }
    pub fn inject_dll<T>(&self, dll: &T) -> crate::Result<()>
    where
        T: AsRef<Path> + ?Sized,
    {
        let dll_path = CString::new(
            dll.as_ref()
                //.canonicalize()?
                .to_str()
                .ok_or(crate::error::Error::OsStrToStdStr)?,
        )?;
        dbg!(&dll_path);
        // let dll_path_pc_str = PCSTR::from_raw(dll_path.as_ptr() as _);
        unsafe {
            let virtual_alloc = self.virtual_alloc(&dll_path)?;

            virtual_alloc.write_process(None)?;

            // GetProcAddress(hmodule, lpprocname)
            let load_thread = CreateRemoteThread(
                self.handle,
                None,
                0,
                /*Some(load_library_a)*/ get_load_library_a()?,
                Some(virtual_alloc.ptr),
                0,
                None,
            )?;

            let _ = WaitForSingleObject(load_thread, INFINITE);

            println!("Dll allocated at : {:p}", virtual_alloc.ptr);
        }
        Ok(())
    }
}
