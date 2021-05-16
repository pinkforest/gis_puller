//! This Rust crate aims to abstract and implement a general global Geographic Information System (GIS) data catalog and acquisition functionality.

#![warn(
  unreachable_pub,
  missing_docs,
  missing_debug_implementations
)]
#![doc(html_logo_url = "TODO.svg")]
#![doc(html_root_url = "https://docs.rs/gis_puller/0.1.0")]
#![doc(issue_tracker_base_url = "https://github.com/pinkforest/rust-git_puller/issues/")]

#[cfg(doctest)]
doc_comment::doctest!("../README.md");

#[macro_use] extern crate quick_error;

//#[macro_use]
//mod macros;

pub mod settings;
pub mod au;


