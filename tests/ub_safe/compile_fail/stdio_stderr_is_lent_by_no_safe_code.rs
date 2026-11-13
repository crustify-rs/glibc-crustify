//! `stderr` names a stream the program's C may close and point elsewhere, as glibc's manual
//! offers, so safe code takes no handle to it: a `'static` one would outlive the stream.

use glibc_bindings::libc::libio::genops::FileRef;
use glibc_bindings::libc::libio::stdio::stderr;

fn main() {
    let _kept: FileRef<'static> = stderr();
}
