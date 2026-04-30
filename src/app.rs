use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::components::{
    draw_central_panel, draw_clear_all_modal, draw_error_modal, draw_footer, draw_inspector_panel,
    draw_tool_bar, draw_top_panel, Colors, CursorHoverState, InspectorTab,
};
use crate::config::AppConfig;
use crate::export::{ExportFormat, ExportService};
use crate::graph::BaseGraph;
use crate::mode::EditMode;
use crate::state::{AppState, UiState};
use crate::update::request_repaint;
use crate::view_state::GraphViewState;

pub struct GraphEditorApp {
    pub state: AppState,
    pub ui: UiState,
    pub export: ExportService,
    pub config: AppConfig,
}

const UI_STATE_STORAGE_KEY: &str = "graph-editor:ui-state";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredUiState {
    version: u32,
    zero_indexed: bool,
    show_number: bool,
    is_directed: bool,
    export_format: String,
}

impl Default for StoredUiState {
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
        let state: StoredUiState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, UI_STATE_STORAGE_KEY))
            .unwrap_or_default();
        app.state.zero_indexed = state.zero_indexed;
        app.state.show_number = state.show_number;
        app.state.graph.is_directed = state.is_directed;
        let format = match state.export_format.as_str() {
            "svg" => ExportFormat::Svg,
            _ => ExportFormat::Png,
        };
        app.export.set_format(format);
        app.sync_input_text_from_graph();
        app
    }

    pub fn deselect_all_vertices_edges(&mut self) {
        for vertex in &mut self.state.graph_view.vertices {
            vertex.is_pressed = false;
            vertex.is_selected = false;
        }
        for edge in &mut self.state.graph_view.edges {
            edge.is_pressed = false;
        }
    }

    pub fn switch_normal_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_normal();
    }

    pub fn switch_add_vertex_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_add_vertex();
    }

    pub fn switch_add_edge_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_add_edge();
    }

    pub fn switch_colorize_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_colorize();
    }

    pub fn switch_delete_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_delete();
    }

    pub fn request_export_image(&mut self, ctx: &egui::Context) {
        let export_ctx = crate::export::ExportContext {
            graph: &self.state.graph,
            view: &self.state.graph_view,
            config: &self.config,
            show_number: self.state.show_number,
            zero_indexed: self.state.zero_indexed,
        };
        if let Some(err) = self.export.request_export(ctx, &export_ctx) {
            self.ui.error_message = Some(err);
        }
    }

    pub fn handle_export_events(&mut self, ctx: &egui::Context) {
        let export_ctx = crate::export::ExportContext {
            graph: &self.state.graph,
            view: &self.state.graph_view,
            config: &self.config,
            show_number: self.state.show_number,
            zero_indexed: self.state.zero_indexed,
        };
        if let Some(err) = self.export.handle_events(ctx, &export_ctx) {
            self.ui.error_message = Some(err);
        }
    }

    pub fn sync_input_text_from_graph(&mut self) {
        let encoded = self.state.graph.encode(self.state.zero_indexed);
        self.ui.input_text = encoded.clone();
        self.ui.input_synced_text = encoded;
        self.ui.input_is_dirty = false;
    }

    pub fn rebuild_from_base_graph(&mut self, ctx: &egui::Context, base_graph: BaseGraph) {
        let canvas_rect = self.ui.canvas_rect.unwrap_or_else(|| ctx.available_rect());
        let visualizer = self.config.visualizer();
        let new_graph_result = self.state.graph.rebuild_from_basegraph(
            visualizer.as_ref(),
            self.config.density_threshold,
            base_graph,
            canvas_rect,
        );
        match new_graph_result {
            Ok(_) => {
                self.state.graph_view.reset_for_graph(&self.state.graph);
                self.state.next_z_index = self.state.graph.vertices.len() as u32;
                self.state.is_animated = true;
                self.sync_input_text_from_graph();
            }
            Err(err) => {
                self.ui.error_message = Some(err.to_string());
            }
        }
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        let graph = crate::graph::Graph::default();
        Self {
            state: AppState {
                graph_view: GraphViewState::new_for_graph(&graph),
                graph,
                is_animated: true,
                last_mouse_pos: None,
                next_z_index: 2,
                edit_mode: EditMode::default_normal(),
                selected_color: Colors::Default,
                zero_indexed: false,
                show_number: true,
            },
            ui: UiState {
                cursor_hover: CursorHoverState::default(),
                canvas_rect: None,
                input_text: String::new(),
                input_synced_text: String::new(),
                input_has_focus: false,
                input_is_dirty: false,
                show_settings: false,
                error_message: None,
                confirm_clear_all: false,
                inspector_tab: InspectorTab::default(),
            },
            export: ExportService::default(),
            config: AppConfig::default(),
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = StoredUiState {
            version: 1,
            zero_indexed: self.state.zero_indexed,
            show_number: self.state.show_number,
            is_directed: self.state.graph.is_directed,
            export_format: self.export.format().extension().to_string(),
        };
        eframe::set_value(storage, UI_STATE_STORAGE_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.graph_view.apply_deletions(&self.state.graph);
        self.state.graph.apply_deletions();

        draw_top_panel(self, ctx);
        draw_tool_bar(self, ctx);
        draw_inspector_panel(self, ctx);
        draw_central_panel(self, ctx);
        draw_footer(self, ctx);
        draw_error_modal(self, ctx);
        draw_clear_all_modal(self, ctx);

        self.handle_export_events(ctx);

        // 再描画
        request_repaint(self, ctx);
    }
}
