//! Archon
//!
//! A library for 


#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![forbid(unsafe_code)]
#![forbid(where_clauses_object_safety)]

/// Archon Telemetry
pub mod telemetry;

/// The core batch submission logic
pub mod batch;

/// The core Archon client
pub mod client;

/// Configuration
pub mod config;

/// Common Archon Errors
pub mod errors;

/// Common internal macros
pub(crate) mod macros;

/// Re-export Archon Types
pub mod prelude {
    pub use crate::batch::*;
    pub use crate::telemetry::*;
    pub use crate::errors::*;
    pub use crate::config::*;
}
