//! A backend-agnostic description of tensor networks, by shape alone.
//!
//! Today it describes one family: an [`Mps`] is a chain of site tensors (a
//! matrix product state), given by the extents of its sites. Nothing here
//! knows how a chain is built, truncated or contracted, and nothing here holds
//! tensor elements, so the same description can be handed to a GPU library,
//! to a CPU library, or to a reference implementation written purely for a
//! test. A method describes its state with it, and a caller compares a
//! requested capacity with the shape a backend produced, without touching
//! the data.
//!
//! It depends only on `thiserror`. This is block ② of
//! `source/TensorNetworkDesign.md`, and the crate's `README.md` explains the
//! model and its choices. When a module or a public type is added, moved or
//! renamed, update the README's "Files" table with it.

mod error;
mod mps;

pub use error::MpsError;
pub use mps::Mps;
