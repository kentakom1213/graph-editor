#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod components;
mod config;
mod graph;
mod mode;
mod update;
mod url;

pub use app::GraphEditorApp;
