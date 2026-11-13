//! The fixture of the suites that write C streams, which no safe constructor yields yet:
//! a stream each test opens and closes through the raw bindings, in each state a C caller
//! can hand one in, and borrows as a [`FileRef`] for as long as it holds it open.

use core::ffi::{CStr, c_char, c_int, c_void};
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::ffi::CString;
use std::io::{self, Read, Write};
use std::os::fd::IntoRawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::{fs, process, thread};

use ffibox::CRefSeam;
use glibc_bindings::libc::ffi;
use glibc_bindings::libc::libio::genops::FileRef;

/// Where an `open_memstream` stream keeps its buffer and size: boxed, since the stream keeps
/// both addresses until it is closed.
struct Location {
    buf: *mut c_char,
    size: usize,
}

/// What a stream writes to, which the fixture keeps until the stream is closed.
enum Backing {
    /// `open_memstream`'s growing buffer.
    Memory(Box<Location>),
    /// `fmemopen`'s fixed storage.
    Fixed(Box<[u8]>),
    /// A file `fopen` opened, removed once the stream is closed.
    File(PathBuf),
    /// `/dev/full`, every write to which fails with `ENOSPC`.
    Full,
    /// The other end of a socket pair, which receives what the stream writes.
    Socket(UnixStream),
}

/// A C stream the test opened, closed when dropped.
pub(crate) struct Stream {
    file: *mut ffi::FILE,
    backing: Backing,
}

impl Stream {
    /// A stream over a buffer that grows, `open_memstream`'s: byte-oriented from its opening.
    pub(crate) fn memory() -> Self {
        let mut location = Box::new(Location {
            buf: ptr::null_mut(),
            size: 0,
        });
        // SAFETY: both out-pointers address the boxed `Location`, which stays at its
        // address until the stream is closed, in `drop`.
        let file = unsafe { ffi::open_memstream(&raw mut location.buf, &raw mut location.size) };
        assert!(!file.is_null(), "open_memstream failed");
        Self {
            file,
            backing: Backing::Memory(location),
        }
    }

    /// A stream over `size` bytes of fixed storage, `fmemopen`'s, opened in `mode`: `"r"`
    /// for one not open for writing, `"w"` for one whose write fails once the storage is
    /// full.
    pub(crate) fn fixed(size: usize, mode: &CStr) -> Self {
        let mut storage = vec![0_u8; size].into_boxed_slice();
        // SAFETY: `storage` holds `size` bytes, and is kept with the stream until it is
        // closed; `mode` is a NUL-terminated string.
        let file =
            unsafe { ffi::fmemopen(storage.as_mut_ptr().cast::<c_void>(), size, mode.as_ptr()) };
        assert!(!file.is_null(), "fmemopen failed");
        Self {
            file,
            backing: Backing::Fixed(storage),
        }
    }

    /// A stream over a new file of the tests' scratch dir, `fopen`'s in `"w"`: unoriented
    /// until its first write, so that `fwide` can orient it for wide characters.
    pub(crate) fn file() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "stream-{}-{}",
            process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let name = CString::new(path.as_os_str().as_bytes()).unwrap();
        // SAFETY: two NUL-terminated strings.
        let file = unsafe { ffi::fopen(name.as_ptr(), c"w".as_ptr()) };
        assert!(!file.is_null(), "fopen failed");
        Self {
            file,
            backing: Backing::File(path),
        }
    }

    /// A stream over `/dev/full`, `fopen`'s in `"w"`, whose buffer's flush fails with
    /// `ENOSPC`.
    pub(crate) fn full() -> Self {
        // SAFETY: two NUL-terminated strings.
        let file = unsafe { ffi::fopen(c"/dev/full".as_ptr(), c"w".as_ptr()) };
        assert!(!file.is_null(), "fopen failed");
        Self {
            file,
            backing: Backing::Full,
        }
    }

    /// A stream over one end of a socket pair, `fdopen`'s in `"r+"`, from which C has read
    /// one byte of the three the other end wrote, leaving two in the stream's buffer: what a
    /// C caller that reads a stream and then writes it without a positioning call, which ISO C
    /// leaves undefined and glibc allows, hands on. glibc seeks back over the unread bytes
    /// before it writes, and a socket cannot seek: every flush fails in that seek, with
    /// `ESPIPE`, before any write, and without setting the error indicator. Made line-buffered
    /// first, as `line_buffered` makes a stream.
    pub(crate) fn line_buffered_socket_after_a_read() -> Self {
        let (ours, mut peer) = UnixStream::pair().unwrap();
        peer.write_all(b"xyz").unwrap();
        peer.set_nonblocking(true).unwrap();
        let fd = ours.into_raw_fd();
        // SAFETY: an open descriptor, which the stream takes and closes, and a NUL-terminated
        // mode.
        let file = unsafe { ffi::fdopen(fd, c"r+".as_ptr()) };
        assert!(!file.is_null(), "fdopen failed");
        let stream = Self {
            file,
            backing: Backing::Socket(peer),
        }
        .line_buffered();
        // SAFETY: an open stream, open for reading.
        assert_eq!(unsafe { ffi::fgetc(stream.file) }, c_int::from(b'x'));
        stream
    }

    /// The stream, borrowed for as long as the fixture keeps it open.
    pub(crate) fn handle(&self) -> FileRef<'_> {
        // SAFETY: `file` is an open stream until `drop` closes it, which the borrow of `self`
        // excludes for the handle's lifetime, and every access the tests make to it is a
        // stdio call that takes its lock, or one they make holding it.
        unsafe { FileRef::from_ptr(self.file) }.unwrap()
    }

    /// The raw stream, for the C calls a test makes on it beside the handle.
    pub(crate) fn as_ptr(&self) -> *mut ffi::FILE {
        self.file
    }

    /// Sets the stream's locking to the caller's, `FSETLOCKING_BYCALLER`, under which
    /// glibc's own stdio routines skip its lock.
    pub(crate) fn lock_by_caller(&self) {
        let bycaller = c_int::try_from(ffi::FSETLOCKING_BYCALLER).unwrap();
        // SAFETY: an open stream, and a locking mode `__fsetlocking` takes.
        unsafe { ffi::__fsetlocking(self.file, bycaller) };
    }

    /// Orients the stream for wide characters, as `fwide(stream, 1)` does.
    pub(crate) fn orient_wide(&self) {
        // SAFETY: an open stream.
        assert!(unsafe { ffi::fwide(self.file, 1) } > 0, "fwide failed");
    }

    /// Makes the stream line-buffered, as `setvbuf(stream, NULL, _IOLBF, BUFSIZ)` does: its
    /// buffer is flushed at each newline. Called before anything else touches the stream, as
    /// `setvbuf` requires.
    pub(crate) fn line_buffered(self) -> Self {
        let mode = c_int::try_from(ffi::_IOLBF).unwrap();
        // SAFETY: an open stream nothing has read or written yet; a NULL buffer, which lets
        // glibc allocate its own, of the size given.
        let r = unsafe { ffi::setvbuf(self.file, ptr::null_mut(), mode, 8192) };
        assert_eq!(r, 0, "setvbuf failed");
        self
    }

    /// Makes the stream unbuffered, as `setvbuf(stream, NULL, _IONBF, 0)` does, as glibc
    /// opens `stderr`: each write goes to the file at once. Called before anything else
    /// touches the stream, as `setvbuf` requires.
    pub(crate) fn unbuffered(self) -> Self {
        let mode = c_int::try_from(ffi::_IONBF).unwrap();
        // SAFETY: an open stream nothing has read or written yet; a NULL buffer and no size,
        // which an unbuffered stream takes.
        let r = unsafe { ffi::setvbuf(self.file, ptr::null_mut(), mode, 0) };
        assert_eq!(r, 0, "setvbuf failed");
        self
    }

    /// The stream's error indicator, as `ferror` reads it.
    pub(crate) fn has_error(&self) -> bool {
        // SAFETY: an open stream.
        unsafe { ffi::ferror(self.file) != 0 }
    }

    /// Clears the stream's error indicator, as `clearerr` does.
    pub(crate) fn clear_error(&self) {
        // SAFETY: an open stream.
        unsafe { ffi::clearerr(self.file) };
    }

    /// The stream's orientation, as `fwide(stream, 0)` reads it: negative for bytes, positive
    /// for wide characters, `0` for none yet.
    pub(crate) fn orientation(&self) -> c_int {
        // SAFETY: an open stream; a mode of `0` only reads the orientation.
        unsafe { ffi::fwide(self.file, 0) }
    }

    /// Whether another thread may take the stream's lock now: `ftrylockfile` from a thread
    /// of its own, released again at once.
    pub(crate) fn is_unlocked(&self) -> bool {
        let file = SendPtr(self.file);
        thread::scope(|s| {
            s.spawn(move || {
                let file = file;
                // SAFETY: an open stream for the scope, which the borrow of `self` keeps.
                let taken = unsafe { ffi::ftrylockfile(file.0) } == 0;
                if taken {
                    // SAFETY: this thread took the lock just above.
                    unsafe { ffi::funlockfile(file.0) };
                }
                taken
            })
            .join()
            .unwrap()
        })
    }

    /// What the stream holds once flushed: an `open_memstream` stream's buffer, the bytes of
    /// an `fmemopen` stream's storage up to its first NUL, the file's, nothing for
    /// `/dev/full`, or what a socket's other end has received.
    pub(crate) fn contents(&self) -> Vec<u8> {
        // SAFETY: an open stream. A flush that fails, as one into full storage does, leaves
        // what reached the storage.
        let flushed = unsafe { ffi::fflush(self.file) };
        match &self.backing {
            Backing::Memory(location) => {
                assert_eq!(flushed, 0, "fflush failed");
                if location.buf.is_null() {
                    return Vec::new();
                }
                // SAFETY: after `fflush`, `open_memstream`'s buffer holds `size` bytes, which
                // nothing writes until the next write to the stream, after this copy.
                unsafe { core::slice::from_raw_parts(location.buf.cast::<u8>(), location.size) }
                    .to_vec()
            }
            Backing::Fixed(storage) => {
                let end = storage
                    .iter()
                    .position(|&b| b == 0)
                    .unwrap_or(storage.len());
                storage[..end].to_vec()
            }
            Backing::File(path) => fs::read(path).unwrap(),
            Backing::Full => Vec::new(),
            Backing::Socket(peer) => {
                let mut received = Vec::new();
                let mut chunk = [0_u8; 4096];
                loop {
                    match (&*peer).read(&mut chunk) {
                        Ok(0) => break,
                        Ok(n) => received.extend_from_slice(&chunk[..n]),
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                        Err(e) => panic!("reading the socket failed: {e}"),
                    }
                }
                received
            }
        }
    }
}

impl Drop for Stream {
    fn drop(&mut self) {
        // SAFETY: an open stream, closed once, here; no handle outlives `self`'s borrow.
        unsafe { ffi::fclose(self.file) };
        match &self.backing {
            Backing::Memory(location) => {
                // SAFETY: `fclose` left `open_memstream`'s buffer to the caller, to free
                // with `free`.
                unsafe { ffi::free(location.buf.cast::<c_void>()) };
            }
            Backing::Fixed(_) | Backing::Full | Backing::Socket(_) => {}
            Backing::File(path) => fs::remove_file(path).unwrap(),
        }
    }
}

/// A raw stream moved into the thread that tries its lock.
#[derive(Clone, Copy)]
struct SendPtr(*mut ffi::FILE);

// SAFETY: the pointer is only passed to `ftrylockfile` and `funlockfile`, which take the
// stream's lock, on a thread that the scope joins while the stream is open.
unsafe impl Send for SendPtr {}
