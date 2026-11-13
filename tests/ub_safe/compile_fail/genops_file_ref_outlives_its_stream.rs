//! A shared handle is borrowed from what keeps its stream open: it cannot outlive that
//! borrow, past which the stream may be closed.

use glibc_bindings::libc::libio::genops::FileRef;

fn keep(stream: FileRef<'_>) -> FileRef<'static> {
    stream
}

fn main() {
    let _ = keep;
}
