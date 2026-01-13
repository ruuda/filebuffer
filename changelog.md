Changelog
=========

1.0.1
-----

Released 2026-01-13.

 * Replace the `rust-crypto` dev-dependency used by the example with `sha2` and
   `hex`. As this only affects the example, this change does not impact users
   of this crate, and there is no need to update.
 * Compatible with Rust 1.40 through 1.93.

Thanks to Alexander Kjäll for contributing to this release.

1.0.0
-----

Released 2024-05-18.

 * **Compatibility:** The minimum supported Rust version is now 1.40, up from
   1.25 previously.
 * Ensure compatibility with Rust 1.78 (which introduced a panic in
   `slice::from_raw_parts` that affects mapping an empty file).
 * Use Rust 2018 edition, upgrade usage of `try!` to the `?` operator.

0.4.0
-----

Released 2018-04-29.

 * Bump `winapi` dependency to 0.3.
 * Ensures compatibility with Rust 1.8 through 1.25 stable.

0.3.0
-----

Released 2017-08-03.

 * Add support for Mac OS X.
 * Implement `AsRef<[u8]>` for `Filebuffer`.
 * Ensures compatibility with Rust 1.8 through 1.19 stable.

Thanks to Craig M. Brandenburg for contributing to this release.

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
