// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

//! This mod contains the platform-specific implementations for Windows based on the winapi crate.

use std::fs;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::ptr;

extern crate kernel32;
extern crate winapi;

pub struct PlatformData {
    // On Windows, the file must be kept open for the lifetime of the mapping.
    #[allow(dead_code)] // The field is not dead, the destructor is effectful.
    file: fs::File,
    mapping_handle: winapi::winnt::HANDLE,
}

impl Drop for PlatformData {
    fn drop (&mut self) {
        if self.mapping_handle != ptr::null_mut() {
            let success = unsafe { kernel32::CloseHandle(self.mapping_handle) };
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

    platform_data.mapping_handle = unsafe { kernel32::CreateFileMappingW(
        file_handle,
        ptr::null_mut(),              // Use default security policy.
        winapi::winnt::PAGE_READONLY, // The memory will be read-only.
        0, 0,                         // The mapping size is the size of the file.
        ptr::null_mut())              // The mapping does not have a name.
    };

    if platform_data.mapping_handle == ptr::null_mut() {
        return Err(io::Error::last_os_error());
    }

    // TODO: Create the view.
    Ok((ptr::null(), 0, platform_data))
}

pub fn unmap_file(buffer: *const u8, length: usize) {
    // TODO: Implement this.
}

/// See also `unix::get_resident`.
pub fn get_resident(buffer: *const u8, length: usize, residency: &mut [bool]) {
    // TODO: Implement this.
}

/// See also `unix::prefetch`.
pub fn prefetch(buffer: *const u8, length: usize) {
    // TODO: Implement this.
}

pub fn get_page_size() -> usize {
    // Fill the `SYSTEM_INFO` struct with zeroes. It will be filled by
    // `GetSystemInfo` later but Rust requires it to be initialized.
    let mut sysinfo = winapi::sysinfoapi::SYSTEM_INFO {
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
    unsafe { kernel32::GetSystemInfo(&mut sysinfo); }
    sysinfo.dwPageSize as usize
}