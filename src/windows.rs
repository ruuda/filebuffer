// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! This mod contains the platform-specific implementations for Windows based on the winapi crate.

use std::fs;
use std::io;
use std::os;
use std::os::windows::io::AsRawHandle;
use std::ptr;

extern crate winapi;

#[derive(Debug)]
pub struct PlatformData {
    // On Windows, the file must be kept open for the lifetime of the mapping.
    #[allow(dead_code)] // The field is not dead, the destructor is effectful.
    file: fs::File,
    mapping_handle: winapi::um::winnt::HANDLE,
}

impl Drop for PlatformData {
    fn drop (&mut self) {
        if self.mapping_handle != ptr::null_mut() {
            let success = unsafe { winapi::um::handleapi::CloseHandle(self.mapping_handle) };
            assert!(success != 0);
        }
    }
}

pub fn map_file(file: fs::File) -> io::Result<(*const u8, usize, PlatformData)> {
    let file_handle = file.as_raw_handle();
    let length = try!(file.metadata()).len();

    if length > usize::max_value() as u64 {
        return Err(io::Error::new(io::ErrorKind::Other, "file is larger than address space"));
    }

    let mut platform_data = PlatformData {
        file: file,
        mapping_handle: ptr::null_mut(),
    };

    // Don't try to map anything if the file is empty.
    if length == 0 {
        return Ok((ptr::null(), 0, platform_data));
    }

    // Memory-mapping a file on Windows is a two-step process: first we create a file mapping
    // object, and then we create a view of that mapping in the virtual address space.
    platform_data.mapping_handle = unsafe {
        winapi::um::memoryapi::CreateFileMappingW(
            file_handle,
            ptr::null_mut(),                  // Use default security policy.
            winapi::um::winnt::PAGE_READONLY, // The memory will be read-only.
            0, 0,                             // The mapping size is the size of the file.
            ptr::null_mut()                   // The mapping does not have a name.
        )
    };

    if platform_data.mapping_handle == ptr::null_mut() {
        return Err(io::Error::last_os_error());
    }

    let result = unsafe {
        winapi::um::memoryapi::MapViewOfFile(
            platform_data.mapping_handle,
            winapi::um::memoryapi::FILE_MAP_READ,     // The memory mapping will be read-only.
            0, 0,                                     // Start offset of the mapping is 0.
            length as winapi::shared::basetsd::SIZE_T // Map the entire file.
        )
    };

    if result == ptr::null_mut() {
        Err(io::Error::last_os_error())
    } else {
        Ok((result as *const u8, length as usize, platform_data))
    }
}

pub fn unmap_file(buffer: *const u8, _length: usize) {
    let success = unsafe {
        winapi::um::memoryapi::UnmapViewOfFile(buffer as *mut os::raw::c_void)
    };
    assert!(success != 0);
}

/// See also `unix::get_resident`.
pub fn get_resident(_buffer: *const u8, _length: usize, residency: &mut [bool]) {
    // As far as I am aware, Windows does not expose a way to query whether pages are resident.
    // There is no equivalent of `mincore()`. The closest thing is `VirtualQuery()`, but the
    // `state` value in the `MEMORY_BASIC_INFORMATION` struct that it fills does not indicate
    // whether the page is resident.

    // Lie and pretend everything is resident.
    for x in residency {
        *x = true;
    }
}

/// See also `unix::prefetch`.
pub fn prefetch(buffer: *const u8, length: usize) {
    let mut entry = winapi::um::memoryapi::WIN32_MEMORY_RANGE_ENTRY {
        VirtualAddress: buffer as *mut os::raw::c_void,
        NumberOfBytes: length as winapi::shared::basetsd::SIZE_T,
    };

    unsafe {
        let current_process_handle = winapi::um::processthreadsapi::GetCurrentProcess();
        winapi::um::memoryapi::PrefetchVirtualMemory(
            current_process_handle, // Prefetch for the current process.
            1, &mut entry,          // An array of length 1 that contains `entry`.
            0                       // Reserved flag that must be 0.
        );
    }

    // The return value of `PrefetchVirtualMemory` is ignored. MSDN says the function may fail if
    // the system is under memory pressure. (It is not entirely clear whether "fail" means
    // "returns a nonzero value", but I assume it does.)
}

pub fn get_page_size() -> usize {
    // Fill the `SYSTEM_INFO` struct with zeroes. It will be filled by
    // `GetSystemInfo` later but Rust requires it to be initialized.
    let mut sysinfo = winapi::um::sysinfoapi::SYSTEM_INFO {
        wProcessorArchitecture: 0,
        wReserved: 0,
        dwPageSize: 0,
        lpMinimumApplicationAddress: ptr::null_mut(),
        lpMaximumApplicationAddress: ptr::null_mut(),
        dwActiveProcessorMask: 0,
        dwNumberOfProcessors: 0,
        dwProcessorType: 0,
        dwAllocationGranularity: 0,
        wProcessorLevel: 0,
        wProcessorRevision: 0
    };
    unsafe { winapi::um::sysinfoapi::GetSystemInfo(&mut sysinfo); }
    sysinfo.dwPageSize as usize
}
