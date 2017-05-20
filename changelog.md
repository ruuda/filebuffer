Changelog
=========

0.2.0
-----

Released 2017-05-20.

 * Derive `fmt::Debug` for public types.
 * Depend on libc only on Unix-like environments, and on kernel32-sys only on
   Windows. This requires Rust 1.8 or later, so this is a breaking change.
 * Ensures compatibility with Rust 1.8 through 1.17 stable.

Thanks to Colin Wallace for contributing to this release.

0.1.1
-----

Released 2017-02-01.

 * Ensures compatibility with Rust 1.4 through 1.14 stable.
 * Host documentation on docs.rs (thanks, docs.rs authors!).
 * Update crate metadata.

0.1.0
-----

Released 2016-01-31.

Initial release with Windows and Linux support.
