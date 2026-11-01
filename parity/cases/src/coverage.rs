//! What each parity operation covers: the Java class it exercises, as a path under
//! `net/minecraft`. `cargo xtask parity report` joins this with the generated cases and lists the
//! operations that still have no entry. Every case compares the Java client with Rust on the same
//! input (`client-parity`); an operation is added here when it gets a Java class, not per case.

/// `(family, operation, Java class)`.
mod exact;
mod inferred_a;
mod inferred_b;
mod inferred_c;

pub use exact::EXACT as COVERAGE;

/// Every inferred list; look an operation up in `COVERAGE` and these.
pub const INFERRED: [&[(&str, &str, &str)]; 3] = [inferred_a::INFERRED, inferred_b::INFERRED, inferred_c::INFERRED];
