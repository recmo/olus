#![doc = include_str!("../Readme.md")]
#![doc(issue_tracker_base_url = "https://github.com/recmo/olus/issues/")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]
#![allow(unused)]

pub mod front;
pub mod parser;

#[cfg(test)]
mod tests {}
