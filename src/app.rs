use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::components::{
    draw_central_panel, draw_clear_all_modal, draw_color_settings, draw_edit_menu,
    draw_error_modal, draw_footer, draw_graph_io, draw_top_panel, Colors, CursorHoverState,
    PanelTabState,
};
use crate::config::AppConfig;
use crate::export::{
    build_export_request, export_color_image, export_svg_bytes, graph_bounds_rect,
    save_export_bytes, ExportFormat, ExportRequest,
};
use crate::graph::Graph;
use crate::mode::EditMode;
use crate::update::request_repaint;

pub struct GraphEditorApp {
    pub graph: Graph,
    pub is_animated: bool,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub next_z_index: u32,
    pub edit_mode: EditMode,
    pub selected_color: Colors,
    pub zero_indexed: bool,
    pub show_number: bool,
    pub cursor_hover: CursorHoverState,
    pub config: AppConfig,
    pub input_text: String,
    pub error_message: Option<String>,
    pub confirm_clear_all: bool,
    pub panel_tab: PanelTabState,
    pub export_format: ExportFormat,
    pub export_in_progress: bool,
    pub export_request: Option<ExportRequest>,
}

const UI_STATE_STORAGE_KEY: &str = "graph-editor:ui-state";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiState {
    version: u32,
    zero_indexed: bool,
    show_number: bool,
    is_directed: bool,
    export_format: String,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            version: 1,
            zero_indexed: false,
            show_number: true,
            is_directed: false,
            export_format: ExportFormat::Png.extension().to_string(),
        }
    }
}

impl GraphEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        let state: UiState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, UI_STATE_STORAGE_KEY))
            .unwrap_or_default();
        app.zero_indexed = state.zero_indexed;
        app.show_number = state.show_number;
        app.graph.is_directed = state.is_directed;
        app.export_format = match state.export_format.as_str() {
            "svg" => ExportFormat::Svg,
            _ => ExportFormat::Png,
        };
        app
    }

    pub fn deselect_all_vertices_edges(&mut self) {
        for vertex in self.graph.vertices_mut() {
            vertex.is_pressed = false;
            vertex.is_selected = false;
        }
        for edge in self.graph.edges_mut() {
            edge.is_pressed = false;
        }
    }

    pub fn switch_normal_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_normal();
    }

    pub fn switch_add_vertex_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_add_vertex();
    }

    pub fn switch_add_edge_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_add_edge();
    }

    pub fn switch_colorize_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_colorize();
    }

    pub fn switch_delete_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_delete();
    }

    pub fn request_export_image(&mut self, ctx: &egui::Context) {
        if self.export_in_progress {
            return;
        }

        let Some(export_request) = build_export_request(self.export_format) else {
            return;
        };

        if export_request.format == ExportFormat::Svg {
            self.export_in_progress = true;
            let result =
                export_svg_bytes(self).and_then(|bytes| save_export_bytes(&export_request, bytes));
            self.export_in_progress = false;
            if let Err(err) = result {
                self.error_message = Some(err.to_string());
            }
            return;
        }

        self.export_in_progress = true;
        self.export_request = Some(export_request);
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
            "graph-export",
        )));
        ctx.request_repaint();
    }

    pub fn handle_export_events(&mut self, ctx: &egui::Context) {
        if self.export_request.is_none() {
            return;
        }

        let screenshot = ctx.input(|i| {
            i.raw.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });

        let Some(screenshot) = screenshot else {
            return;
        };

        let export_request = self.export_request.take();
        self.export_in_progress = false;

        let Some(export_request) = export_request else {
            return;
        };

        let pixels_per_point = ctx.pixels_per_point();
        let Some(mut region) = graph_bounds_rect(self) else {
            self.error_message = Some("Export failed: no vertices to capture.".to_string());
            return;
        };
        region = region.intersect(ctx.screen_rect());

        if region.width() <= 0.0 || region.height() <= 0.0 {
            self.error_message = Some("Export failed: invalid capture region.".to_string());
            return;
        }

        let mut color_image = screenshot.region(&region, Some(pixels_per_point));

        if color_image.width() == 0 || color_image.height() == 0 {
            self.error_message = Some("Export failed: empty capture region.".to_string());
            return;
        }

        let result = export_color_image(&mut color_image)
            .and_then(|bytes| save_export_bytes(&export_request, bytes));

        if let Err(err) = result {
            self.error_message = Some(err.to_string());
        }
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            is_animated: true,
            last_mouse_pos: None,
            next_z_index: 2,
            edit_mode: EditMode::default_normal(),
            selected_color: Colors::Default,
            zero_indexed: false,
            show_number: true,
            cursor_hover: CursorHoverState::default(),
            config: AppConfig::default(),
            input_text: String::new(),
            error_message: None,
            confirm_clear_all: false,
            panel_tab: PanelTabState::default(),
            export_format: ExportFormat::Png,
            export_in_progress: false,
            export_request: None,
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = UiState {
            version: 1,
            zero_indexed: self.zero_indexed,
            show_number: self.show_number,
            is_directed: self.graph.is_directed,
            export_format: self.export_format.extension().to_string(),
        };
        eframe::set_value(storage, UI_STATE_STORAGE_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // トップパネル（タブバー）を描画
        draw_top_panel(self, ctx);

        // メイン領域を描画
        draw_central_panel(self, ctx);

        // 現在選択されているタブに応じてサイドパネルの内容を切り替える
        if self.panel_tab.edit_menu {
            // 編集メニューを描画
            draw_edit_menu(self, ctx);
        }
        if self.panel_tab.color_settings {
            // 色の設定を描画
            draw_color_settings(self, ctx);
        }
        if self.panel_tab.graph_io {
            // グラフの入力を描画
            draw_graph_io(self, ctx);
        }

        // フッターを描画
        draw_footer(self, ctx);

        // エラーメッセージを描画
        draw_error_modal(self, ctx);
        draw_clear_all_modal(self, ctx);

        self.handle_export_events(ctx);

        // 再描画
        request_repaint(self, ctx);
    }
}
