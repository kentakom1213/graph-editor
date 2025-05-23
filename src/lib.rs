#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod components;
mod config;
mod graph;
mod math;
mod mode;
mod update;

pub use app::GraphEditorApp;
