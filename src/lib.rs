#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod components;
mod config;
mod export;
mod graph;
mod math;
mod mode;
mod state;
mod update;
mod view_state;

pub use app::GraphEditorApp;
