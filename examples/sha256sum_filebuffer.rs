// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

// This example implements the `sha256sum` program in Rust using the Filebuffer library. Compare
// with `sha256sum_naive` which uses the IO primitives in the standard library.

use std::env;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use filebuffer::FileBuffer;

extern crate crypto;
extern crate filebuffer;

fn main() {
    for fname in env::args().skip(1) {
        let fbuffer = FileBuffer::open(&fname).expect("failed to open file");
        let mut hasher = Sha256::new();
        hasher.input(&fbuffer);

        // Match the output format of `sha256sum`, which has two spaces between the hash and name.
        println!("{}  {}", hasher.result_str(), fname);
    }
}
