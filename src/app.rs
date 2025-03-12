use eframe::egui;

use crate::components::{draw_central_panel, draw_edit_menu, draw_footer, draw_graph_input};
use crate::config::AppConfig;
use crate::graph::Graph;
use crate::mode::EditMode;

pub struct GraphEditorApp {
    pub graph: Graph,
    pub next_z_index: u32,
    pub edit_mode: EditMode,
    pub zero_indexed: bool,
    pub hovered_on_menu_window: bool,
    pub hovered_on_input_window: bool,
    pub config: AppConfig,
}

impl GraphEditorApp {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            next_z_index: 2,
            edit_mode: EditMode::default_normal(),
            zero_indexed: false,
            hovered_on_menu_window: false,
            hovered_on_input_window: false,
            config: AppConfig::default(),
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // メイン領域を描画
        draw_central_panel(self, ctx);

        // 編集メニューを描画
        draw_edit_menu(self, ctx);

        // グラフの入力を描画
        draw_graph_input(self, ctx);

        // フッターを描画
        draw_footer(self, ctx);
    }
}
