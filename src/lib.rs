#![doc = include_str!("../Readme.md")]
#![doc(issue_tracker_base_url = "https://github.com/recmo/olus/issues/")]

mod diagnostic;
mod files;
pub mod front;
pub mod ir;

pub use crate::{
    diagnostic::Diagnostic,
    files::{FileId, Files, Span},
};

#[cfg(test)]
mod tests {}
