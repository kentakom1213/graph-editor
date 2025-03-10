use eframe::egui;

use crate::components::{draw_central_panel, draw_edit_menu, draw_footer, draw_graph_input};
use crate::config::AppConfig;
use crate::graph::Graph;
use crate::mode::EditMode;

pub struct GraphEditorApp {
    pub(crate) graph: Graph,
    pub(crate) next_z_index: u32,
    pub(crate) edit_mode: EditMode,
    pub(crate) zero_indexed: bool,
    pub(crate) config: AppConfig,
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
        draw_footer(ctx);
    }
}
