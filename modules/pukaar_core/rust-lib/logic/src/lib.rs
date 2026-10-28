//! Pukaar's rules, with no Logos dependency: events, per-author chains, the
//! reducer, anti-entropy and checkpoints. The Logos module in ../rust-lib wraps `Node`.

/// The rules crate, shared with the program: the module uses its types through this name.
pub use pukaar_sites_rules as rules;

pub mod chain;
pub mod checkpoint;
pub mod event;
pub mod fetch;
pub mod identity;
pub mod joinreq;
pub mod lezkey;
pub mod meet;
pub mod node;
pub mod ops;
pub mod persist;
pub mod profile;
pub mod queue;
pub mod reducer;
pub mod rpc;
pub mod store;
pub mod sync;
pub mod writes;
