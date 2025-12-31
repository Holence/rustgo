#![allow(
    clippy::needless_return,
    clippy::redundant_field_names,
    clippy::collapsible_if,
    clippy::collapsible_else_if
)]
pub mod board;
pub mod common;
pub mod model;
pub mod player;
pub mod view;

pub use common::*;
