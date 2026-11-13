//! Equivalence tests of the exported API against the C it wraps.
#![expect(
    unsafe_code,
    reason = "reads and writes the C globals the wrappers stand in for, and calls the C they wrap"
)]

mod stdio;
