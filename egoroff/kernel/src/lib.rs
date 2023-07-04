#![warn(unused_extern_crates)]
#![warn(clippy::unwrap_in_result)]
#![warn(clippy::unwrap_used)]
#![allow(clippy::missing_errors_doc)]

pub mod archive;
pub mod converter;
pub mod domain;
pub mod graph;
pub mod resource;
pub mod session;
pub mod sqlite;
pub mod typograph;
pub mod xml;

#[macro_use]
extern crate serde;
