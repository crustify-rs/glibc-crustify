//! Equivalence tests of the exported API against the C it wraps.
#![expect(unsafe_code, reason = "reads the C globals the wrappers stand in for")]

mod stdio;
