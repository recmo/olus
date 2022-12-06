#![doc = include_str!("../Readme.md")]
#![doc(issue_tracker_base_url = "https://github.com/recmo/olus/issues/")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(unused)]

mod diagnostic;
mod files;
pub mod front;
pub mod names;
pub mod parser;

pub use crate::{
    diagnostic::Diagnostic,
    files::{FileId, Files, Span},
};

#[cfg(test)]
mod tests {}
