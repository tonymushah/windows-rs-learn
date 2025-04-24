use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    path::Path,
};

use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::Debug::WriteProcessMemory,
        LibraryLoader::LoadLibraryA,
        Memory::{
            MEM_COMMIT, MEM_RELEASE, PAGE_PROTECTION_FLAGS, PAGE_READWRITE,
            VIRTUAL_ALLOCATION_TYPE, VirtualAllocEx, VirtualFreeEx,
        },
        Threading::{
            CreateRemoteThread, INFINITE, OpenProcess, PROCESS_ALL_ACCESS, WaitForSingleObject,
        },
    },
};
use windows_core::{Owned, PCSTR};

use crate::process::get_process_id;

pub struct Process {
    handle: Owned<HANDLE>,
}

unsafe extern "system" fn load_library_a(parameter: *mut c_void) -> u32 {
    match unsafe { LoadLibraryA(PCSTR(parameter as _)) } {
        Ok(_mod) => 0,
        Err(err) => err.code().0 as _,
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
                *process.handle,
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
                *self.process.handle,
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
                *self.process.handle,
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
        Ok(Self {
            handle: unsafe { Owned::new(handle) },
        })
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
                .canonicalize()?
                .to_str()
                .ok_or(crate::error::Error::OsStrToStdStr)?,
        )?;
        // let dll_path_pc_str = PCSTR::from_raw(dll_path.as_ptr() as _);
        unsafe {
            let virtual_alloc = self.virtual_alloc(&dll_path)?;

            virtual_alloc.write_process(None)?;
            let load_thread = CreateRemoteThread(
                *self.handle,
                None,
                0,
                Some(load_library_a),
                Some(virtual_alloc.ptr),
                0,
                None,
            )?;

            let _ = WaitForSingleObject(load_thread, INFINITE);

            println!("Dll allocated at : {}", virtual_alloc.ptr.addr());
        }
        Ok(())
    }
}
