use std::os::raw::c_ulong;

use windows::Win32::{
    Foundation::HANDLE,
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, HEAPENTRY32, HEAPENTRY32_FLAGS, HEAPLIST32, Heap32First,
            Heap32ListFirst, Heap32ListNext, Heap32Next, TH32CS_SNAPHEAPLIST,
        },
        Memory::{GetProcessHeaps, HeapWalk, PROCESS_HEAP_ENTRY},
        SystemServices::PROCESS_HEAP_REGION,
        Threading::GetCurrentProcessId,
    },
};
use windows_core::Owned;

#[derive(Debug, Clone, Copy)]
pub struct HeapEntry {
    pub heap_id: usize,
    pub handle: HANDLE,
    pub address: usize,
    pub block_size: usize,
    pub flags: HEAPENTRY32_FLAGS,
}

impl From<HEAPENTRY32> for HeapEntry {
    fn from(value: HEAPENTRY32) -> Self {
        Self {
            heap_id: value.th32HeapID,
            handle: value.hHandle,
            address: value.dwAddress,
            block_size: value.dwBlockSize,
            flags: value.dwFlags,
        }
    }
}

pub fn get_current_process_heap_handles() -> windows_core::Result<Vec<HANDLE>> {
    let mut actual_count = 0u32;
    let mut maybe_heaps = Vec::new();
    loop {
        let count = unsafe { GetProcessHeaps(&mut maybe_heaps) };
        if count <= actual_count {
            break;
        }
        actual_count = count;
        maybe_heaps.resize_with(actual_count as usize, Default::default);
    }
    Ok(maybe_heaps)
}

pub fn get_current_process_heaps() -> windows_core::Result<Vec<HeapEntry>> {
    let mut entries = Vec::new();
    let handles = get_current_process_heap_handles()?;
    for handle in handles {
        let mut entry = PROCESS_HEAP_ENTRY {
            ..Default::default()
        };
        let heap_id = unsafe { handle.0.cast::<c_ulong>().read() };
        while unsafe { HeapWalk(handle, &mut entry).is_ok() } {
            if entry.wFlags != (PROCESS_HEAP_REGION as u16) {
                entries.push(HeapEntry {
                    heap_id: heap_id as usize,
                    handle,
                    address: handle.0.addr(),
                    block_size: (entry.cbData + (entry.cbOverhead as u32)) as usize,
                    flags: HEAPENTRY32_FLAGS(entry.wFlags.into()),
                });
            }
        }
    }
    Ok(entries)
}

pub fn get_heap_entries(process: Option<u32>) -> windows_core::Result<Vec<HeapEntry>> {
    let process_id = process.unwrap_or(unsafe { GetCurrentProcessId() });
    let dbg_handle =
        unsafe { Owned::new(CreateToolhelp32Snapshot(TH32CS_SNAPHEAPLIST, process_id)?) };
    let mut heap_entries = Vec::<HeapEntry>::new();
    {
        let mut current = HEAPLIST32 {
            dwSize: size_of::<HEAPLIST32>(),
            ..Default::default()
        };
        unsafe { Heap32ListFirst(*dbg_handle, &mut current)? };
        loop {
            let mut entry = HEAPENTRY32 {
                dwSize: size_of::<HEAPENTRY32>(),
                ..Default::default()
            };
            unsafe {
                if Heap32First(&mut entry, process_id, current.th32HeapID).is_err() {
                    continue;
                }
            }
            loop {
                heap_entries.push(entry.into());
                unsafe {
                    if Heap32Next(&mut entry).is_err() {
                        break;
                    }
                }
            }
            unsafe {
                if Heap32ListNext(*dbg_handle, &mut current).is_err() {
                    break;
                }
            };
        }
    }
    Ok(heap_entries)
}
