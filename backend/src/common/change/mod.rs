//! # CRDT Change File System
//!
//! A CRDT-based distributed change file system supporting multi-agent concurrent collaboration.
//!
//! ## Modules
//!
//! - [`change`] - Core Change data structure
//! - [`operation`] - Operation types for different changes
//! - [`version`] - Vector clock (version) for causality tracking
//! - [`thread`] - Thread management (fork, merge)
//! - [`merge`] - CRDT merge engine
//! - [`snapshot`] - Snapshot generation from change sequences

pub mod change;
pub mod merge;
pub mod operation;
pub mod snapshot;
pub mod thread;
pub mod version;

// Re-export main types for convenience
pub use change::Change;
pub use merge::MergeEngine;
pub use operation::Operation;
pub use snapshot::Snapshot;
pub use thread::Thread;
pub use version::VectorClock;
