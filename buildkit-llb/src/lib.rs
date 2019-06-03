#![feature(existential_type)]
#![deny(warnings)]
#![deny(clippy::all)]

// FIXME: get rid of the unwraps
// TODO: support cache disabling for each operation.

mod serialization;

/// Supported operations - building blocks of the LLB definition graph.
pub mod ops;

/// Various helpers and types.
pub mod utils;

/// Convenient re-export of a commonly used things.
pub mod prelude {
    pub use crate::ops::*;
    pub use crate::utils::{OutputIdx, OwnOutputIdx};
}
