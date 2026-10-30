#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

pub mod blog;
pub mod converter;
pub mod database;
pub mod domain;
pub mod graph;
#[cfg(test)]
mod lock_test;
pub mod paging;
pub mod resource;
pub mod session;
pub mod sqlite;
pub mod typograph;
pub mod xml;

#[macro_use]
extern crate serde;
