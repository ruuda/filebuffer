// Filebuffer -- Fast and simple file reading
// Copyright 2016 Ruud van Asseldonk
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// A copy of the License has been included in the root of the repository.

// This example implements the `head` program in Rust using the Filebuffer library.
// Input files are assumed to be valid UTF-8.

use std::env;
use std::str;
use filebuffer::FileBuffer;

extern crate filebuffer;

fn main() {
    for fname in env::args().skip(1) {
        println!("==> {} <==", &fname);
        let fbuffer = FileBuffer::open(&fname).expect("failed to open file");
        let lines = str::from_utf8(&fbuffer).expect("not valid UTF-8").lines();
        for line in lines.take(10) {
            println!("{}", line);
        }
    }
}
