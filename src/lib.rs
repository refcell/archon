//! Archon
//!
//! A batch submission library for generalized rollups.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

/// Archon Telemetry
pub mod telemetry;

/// The core batch submission logic
pub mod driver;

/// The rollup node
pub mod rollup;

/// The core Archon client
pub mod client;

/// Configuration
pub mod config;

/// Common Archon Errors
pub mod errors;

/// Channel Manager
pub mod channels;

/// The channel builder
pub mod builder;

/// The transaction manager
pub mod transactions;

/// Common internal macros
pub(crate) mod macros;

/// Re-export Archon Types
pub mod prelude {
    pub use crate::config::*;
    pub use crate::errors::*;
    pub use crate::telemetry::*;

    /// Re-export driver-related types.
    ///
    /// The [crate::driver::Driver] is responsible for polling the L1 chain
    /// for the latest [Block] and feeding it's [BlockId] back to [Archon].
    pub use crate::driver::*;

    /// Re-export rollup-related types.
    pub use crate::rollup::*;

    /// Re-export transaction manager related types.
    pub use crate::transactions::*;

    pub use crate::builder::*;
    /// Re-export channel-related types.
    pub use crate::channels::*;
}
