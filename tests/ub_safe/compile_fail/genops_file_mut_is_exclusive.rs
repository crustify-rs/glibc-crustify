//! The exclusive handle is not copied: two of one stream would alias.

use glibc_bindings::libc::libio::genops::FileMut;

fn twice(stream: FileMut<'_>) -> (FileMut<'_>, FileMut<'_>) {
    (stream, stream)
}

fn main() {
    let _ = twice;
}
