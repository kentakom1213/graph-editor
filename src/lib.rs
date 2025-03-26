#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod components;
mod config;
mod graph;
mod mode;
mod update_paint;

pub use app::GraphEditorApp;
