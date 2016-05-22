Filebuffer
==========
Fast and simple file reading for Rust.

[![Build Status][tr-img]][tr]
[![Build Status][av-img]][av]
[![Crates.io version][crate-img]][crate]
[![Documentation][docs-img]][docs]

Filebuffer can map files into memory. This is ofter faster than using the
primitives in `std::io`, and also more convenient. Furthermore this crate
offers prefetching and checking whether file data is resident in physical
memory (so access will not incur a page fault). This enables non-blocking
file reading.

Example
-------
Below is an implementation of the `sha256sum` program that is both faster and
simpler than the naive `std::io` equivalent. (See `sha256sum_filebuffer` and
`sha256sum_naive` in the examples directory.)

```rust
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
        println!("{}  {}", hasher.result_str(), fname);
    }
}
```

License
-------
Filebuffer is licensed under the [Apache 2.0][apache2] license. It may be used
in free software as well as closed-source applications, both for commercial and
non-commercial use under the conditions given in the license. If you want to use
Filebuffer in your GPLv2-licensed software, you can add an [exception][except]
to your copyright notice.

[tr-img]:    https://travis-ci.org/ruuda/filebuffer.svg?branch=master
[tr]:        https://travis-ci.org/ruuda/filebuffer
[av-img]:    https://ci.appveyor.com/api/projects/status/kfacq5ul22hbnd9u?svg=true
[av]:        https://ci.appveyor.com/project/ruuda/filebuffer
[crate-img]: http://img.shields.io/crates/v/filebuffer.svg
[crate]:     https://crates.io/crates/filebuffer
[docs-img]:  http://img.shields.io/badge/docs-online-blue.svg
[docs]:      https://ruuda.github.io/filebuffer/doc/v0.1.0/filebuffer/
[apache2]:   https://www.apache.org/licenses/LICENSE-2.0
[except]:    https://www.gnu.org/licenses/gpl-faq.html#GPLIncompatibleLibs
