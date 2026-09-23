//! Pukaar's rules, with no Logos dependency: events, per-author chains, the
//! reducer, anti-entropy and checkpoints. The Logos module in ../rust-lib wraps `Node`.

pub mod checkpoint;
pub mod event;
pub mod node;
pub mod reducer;
pub mod store;
pub mod sync;
